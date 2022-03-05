use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{convert::Infallible, net::SocketAddr};

use hyper::Server;
use hyper::{
    body::Body,
    http::{Request, Response},
    service::{make_service_fn, service_fn},
};
use inspect::{Inspect, InspectOk};
use sai::{Component, ComponentLifecycle, Injected};
use tokio::sync::oneshot;
use util::elapse;

use crate::config::Config;
use crate::model::{Model, Presenter};
use crate::msg::Msg;
use crate::repository::RepositorySet;
use crate::usecase::{get_book, get_books, get_books_by_ids};

#[derive(Component)]
pub struct Resolver {
    #[injected]
    repository: Injected<RepositorySet>,
    // #[injected]
    // command: Injected<CommandSet>,
    // #[injected]
    // config: Injected<Config>,
}

impl Resolver {
    async fn resolve(&self, msg: Msg) -> crate::Result<Model> {
        let repository = Arc::clone(&self.repository);
        // let command = Arc::clone(&self.command);
        // let config = Arc::clone(&self.config);

        let model = match msg {
            Msg::GetBooks(payload) => get_books::execute(payload, repository).await?.into(),

            Msg::GetBooksByIds(payload) => {
                get_books_by_ids::execute(payload, repository).await?.into()
            }

            Msg::GetBook(payload) => get_book::execute(payload, repository).await?.into(),
        };

        Ok(model)
    }
}

#[derive(Component)]
#[lifecycle]
pub struct HttpServer {
    #[injected]
    resolver: Injected<Resolver>,
    /* tx: Option<mpsc::Sender<()>>,
    rx: Option<mpsc::Receiver<()>>, */
    #[injected]
    config: Injected<Config>,

    stop_sender: Option<oneshot::Sender<()>>,

    stopped_reciever: Option<oneshot::Receiver<()>>,
}

async fn handler(
    request: &mut Request<Body>,
    response: &mut Response<Body>,
    resolver: Arc<Resolver>,
    config: Arc<Config>,
) -> crate::Result<()> {
    let msg = elapse!("route", Msg::http(request, response, config.clone()).await?);

    let model = elapse!("execute", resolver.resolve(msg).await?);

    let _r = elapse!(
        "present",
        model.set_response(request, response, config).await?
    );

    Ok(())
}

async fn service(
    mut request: Request<Body>,
    resolver: Arc<Resolver>,
    config: Arc<Config>,
) -> Result<Response<Body>, Infallible> {
    let req_method = request.method().to_owned();
    let req_uri = request.uri().to_string();

    log::info!("--> {} {}", req_method, req_uri);

    let start = SystemTime::now();

    let mut response = Response::new(Body::empty());
    let ret = elapse!(
        "handle",
        handler(&mut request, &mut response, resolver, config.clone()).await
    );

    let end = start
        .elapsed()
        .as_ref()
        .map(Duration::as_micros)
        .unwrap_or(0);

    if let Err(err) = ret {
        err.inspect(|e| log::error!("{}", e))
            .set_response(&mut request, &mut response, config)
            .await
            .expect("in err.set_response()");
    }

    Ok(response).inspect_ok(|res| {
        log::info!(
            "<-- {} {} {} {}ms",
            req_method,
            req_uri,
            res.status(),
            end as f64 / 1000.0
        )
    })
}

#[async_trait::async_trait]
impl ComponentLifecycle for HttpServer {
    async fn start(&mut self) {
        let (stop_tx, stop_rx) = oneshot::channel();
        let (stopped_tx, stopped_rx) = oneshot::channel();

        self.stop_sender.replace(stop_tx);
        self.stopped_reciever.replace(stopped_rx);

        let resolver = Arc::clone(&self.resolver);
        let config = Arc::clone(&self.config);

        let port = self.config.port();

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            let svc = |resolver: Arc<Resolver>, config: Arc<Config>| async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    service(request, resolver.clone(), config.clone())
                }))
            };

            let server = Server::bind(&addr).serve(make_service_fn(move |_| {
                svc(resolver.clone(), config.clone())
            }));

            let server = Server::with_graceful_shutdown(server, async {
                stop_rx.await.unwrap();
            });

            log::info!("started http server: 0.0.0.0:{}", port);

            if let Err(err) = server.await {
                log::error!("{:?}", err);
            }

            stopped_tx.send(()).unwrap();
        });
    }

    async fn stop(&mut self) {
        let stop_tx = self.stop_sender.take().unwrap();

        stop_tx.send(()).unwrap();

        let stopped_rx = self.stopped_reciever.take().unwrap();

        stopped_rx.await.unwrap();
    }
}

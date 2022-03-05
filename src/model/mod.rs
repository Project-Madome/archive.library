mod book;

pub use book::Book;

use std::sync::Arc;

use hyper::{header, Body, Request, Response, StatusCode};
use util::http::SetResponse;

use crate::{config::Config, into_model, model};

into_model![(Book, Book), (Books, Vec<Book>)];

#[async_trait::async_trait]
pub trait Presenter: Sized {
    /// &mut Request를 받는 이유는 핸들러에서 body parse하는 과정에서 mutable이 필요하기 때문임
    async fn set_response(
        self,
        request: &mut Request<Body>,
        response: &mut Response<Body>,
        config: Arc<Config>,
    ) -> crate::Result<()>;
}

#[macro_export]
macro_rules! into_model {
    () => {
        pub enum Model {}
    };

    ($(($member:ident, $from:ty)),*$(,)?) => {
        pub enum Model {
            $(
                $member($from),
            )*
        }

        $(
            impl From<$from> for Model {
                fn from(from: $from) -> Model {
                    Model::$member(from)
                }
            }
        )*


        #[async_trait::async_trait]
        impl Presenter for Model {
            async fn set_response(self, request: &mut Request<Body>, response: &mut Response<Body>, config: Arc<Config>) -> crate::Result<()> {
                use Model::*;

                match self {
                    $(
                        $member(model) => model.set_response(request, response, config).await,
                    )*
                }
            }
        }

    };
}

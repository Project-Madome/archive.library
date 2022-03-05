use std::sync::Arc;

use hyper::{header, Body, Method, Request, Response};

use madome_sdk::api::auth;
use parking_lot::RwLock;
use serde::Deserialize;
use util::{
    elapse,
    http::{
        url::{is_path_variable, PathVariable},
        SetResponse,
    },
};

use crate::{
    config::Config,
    payload,
    usecase::{get_book, get_books, get_books_by_ids},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
}

/// Msg의 Payload는 같은 이름의 usecase의 Payload와는 관계가 없음
///
/// Msg의 Payload는 실행되어야하는 usecase 순서에 따라 정해짐 (제일 처음 실행하는 usecase의 Payload)
///
/// 실행되는 순서는 Resolver 참조
#[derive(Debug)]
pub enum Msg {
    GetBooks(get_books::Payload),
    GetBook(get_book::Payload),
    GetBooksByIds(get_books_by_ids::Payload),
}

impl Msg {
    pub async fn http(
        request: &mut Request<Body>,
        resp: &mut Response<Body>,
        config: Arc<Config>,
    ) -> crate::Result<Self> {
        let headers = request.headers();

        /* let cookie = Cookie::from(headers);

        let access_token = cookie.get(MADOME_ACCESS_TOKEN).unwrap_or_default();
        let refresh_token = cookie.get(MADOME_REFRESH_TOKEN).unwrap_or_default(); */

        if let Some(cookie) = headers.get(header::COOKIE).cloned() {
            resp.set_header(header::COOKIE, cookie).unwrap();
        }

        // 외부 사용자의 요청일 경우에는 토큰 인증을 함
        if auth::check_internal(headers).is_err() {
            let resp = RwLock::new(resp);

            let _r = elapse!(
                "check_auth",
                auth::check_and_refresh_token_pair(config.auth_url(), &resp, None).await?
            );
        }

        let method = request.method().clone();
        let path = request.uri().path();

        let msg = match (method, path) {
            (Method::GET, "/books") => {
                let is_get_books_by_ids =
                    request.uri().query().unwrap_or_default().contains("ids[]=");

                if is_get_books_by_ids {
                    Msg::GetBooksByIds(request.try_into()?)
                } else {
                    Msg::GetBooks(request.try_into()?)
                }
            }

            (Method::GET, path) if matcher(path, "/books/:book_id") => {
                Msg::GetBook(PathVariable::new(path, "/books/:book_id").try_into()?)
            }

            _ => return Err(Error::NotFound.into()),
        };

        log::debug!("msg = {msg:?}");

        Ok(msg)
    }
}

fn matcher(req_path: &str, pattern: &str) -> bool {
    let mut origin = req_path.split('/');
    let pats = pattern.split('/');

    for pat in pats {
        if let Some(origin) = origin.next() {
            if !is_path_variable(pat) && pat != origin {
                return false;
            }
        } else {
            return false;
        }
    }

    origin.next().is_none()
}

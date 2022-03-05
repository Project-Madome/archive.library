use std::sync::Arc;

use chrono::{DateTime, Utc};
use hyper::{header, Body, Request, Response, StatusCode};
use serde::Serialize;
use util::{elapse, http::SetResponse};

use crate::{config::Config, entity};

use super::Presenter;

#[derive(Debug, Serialize)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub kind: String,
    pub page: usize,
    pub language: String,
    pub tags: Vec<(String, String)>,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
impl Presenter for Book {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        resp: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        let serialized = elapse!(
            "serialize",
            serde_json::to_vec(&self).expect("json serialize")
        );

        resp.set_status(StatusCode::OK).unwrap();
        resp.set_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        resp.set_body(serialized.into());

        Ok(())
    }
}

#[async_trait::async_trait]
impl Presenter for Vec<Book> {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        resp: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        let serialized = elapse!(
            "serialize",
            serde_json::to_vec(&self).expect("json serialize")
        );

        resp.set_status(StatusCode::OK).unwrap();
        resp.set_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        resp.set_body(serialized.into());

        Ok(())
    }
}

impl From<entity::Book> for Book {
    fn from(
        entity::Book {
            id,
            title,
            kind,
            page,
            language,
            tags,
            created_at,
        }: entity::Book,
    ) -> Self {
        let tags = tags
            .into_iter()
            .map(|x| (x.kind().to_owned(), x.name().to_owned()))
            .collect();

        Self {
            id,
            title,
            kind: kind.as_str().to_owned(),
            page,
            language,
            tags,
            created_at,
        }
    }
}

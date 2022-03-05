use std::sync::Arc;

use hyper::{Body, Request};
use itertools::Itertools;
use serde::Deserialize;
use util::{validate::ValidatorNumberExt, MapInto};

use crate::{
    error::UseCaseError,
    model, payload,
    repository::{r#trait::BookRepository, RepositorySet},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Payload {
    pub kind: Option<payload::BookKind>,
    pub per_page: Option<usize>,
    pub page: Option<usize>,
    pub sort_by: Option<payload::BookSortBy>,
}

impl Payload {
    pub fn check(self) -> crate::Result<Self> {
        let per_page = self
            .per_page
            .unwrap_or(25)
            .validate()
            .min(1)
            .max(100)
            .take()
            .map_err(payload::Error::InvalidPerPage)?;

        let page = self
            .page
            .unwrap_or(1)
            .validate()
            .min(1)
            .take()
            .map_err(payload::Error::InvalidPage)?;

        let sort_by = self.sort_by.unwrap_or(payload::BookSortBy::IdDesc);

        Ok(Self {
            kind: self.kind,
            per_page: Some(per_page),
            page: Some(page),
            sort_by: Some(sort_by),
        })
    }
}

impl TryFrom<&mut Request<Body>> for Payload {
    type Error = crate::Error;

    fn try_from(request: &mut Request<Body>) -> Result<Self, Self::Error> {
        let qs = request.uri().query().unwrap_or_default();
        let payload: Payload =
            serde_qs::from_str(qs).map_err(payload::Error::QuerystringDeserialize)?;

        payload.check()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub type Model = Vec<model::Book>;

pub async fn execute(
    Payload {
        kind,
        per_page,
        page,
        sort_by,
    }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let books = repository
        .book()
        .get_many(
            kind.map_into(),
            per_page.unwrap(),
            page.unwrap(),
            sort_by.map_into().unwrap(),
        )
        .await?;

    Ok(books.into_iter().map_into().collect())
}

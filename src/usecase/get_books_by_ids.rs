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
    pub ids: Vec<u32>,
}

impl Payload {
    pub fn check(self) -> crate::Result<Self> {
        if self.ids.len() > 100 {
            return Err(
                payload::Error::Custom("length of ids must be less than or equal 100").into(),
            );
        }

        Ok(self)
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
    Payload { ids }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let books = repository.book().get_many_by_ids(ids).await?;

    Ok(books.into_iter().map_into().collect())
}

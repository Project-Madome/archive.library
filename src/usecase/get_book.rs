use std::sync::Arc;

use util::http::url::PathVariable;

use crate::{
    error::UseCaseError,
    model, payload,
    repository::{r#trait::BookRepository, RepositorySet},
};

#[derive(Debug)]
pub struct Payload {
    pub book_id: u32,
}

impl TryFrom<PathVariable> for Payload {
    type Error = crate::Error;

    fn try_from(mut path_var: PathVariable) -> Result<Self, Self::Error> {
        match path_var.next_variable::<u32>() {
            Some(book_id) => Ok(Payload { book_id }),
            None => Err(payload::Error::InvalidPathVariable("book_id", "number").into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found book")]
    NotFoundBook,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub type Model = model::Book;

pub async fn execute(
    Payload { book_id }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let book = repository
        .book()
        .get_one(book_id)
        .await?
        .ok_or(Error::NotFoundBook)?;

    Ok(book.into())
}

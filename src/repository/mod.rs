mod inmemory;
mod postgresql;
pub mod r#trait;

pub use inmemory::*;
pub use postgresql::*;

use std::sync::Arc;

use sai::{Component, Injected};

#[derive(Component)]
pub struct RepositorySet {
    #[injected]
    book_repository: Injected<PostgresqlBookRepository>,
}

impl RepositorySet {
    pub fn book(&self) -> Arc<impl r#trait::BookRepository> {
        Arc::clone(&self.book_repository)
    }
}

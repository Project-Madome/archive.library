use crate::entity::{Book, BookGroupByTag, BookKind, BookSortBy, BookTag};

#[async_trait::async_trait]
pub trait BookRepository: Send + Sync {
    async fn get_one(&self, book_id: u32) -> crate::Result<Option<Book>>;

    async fn get_many(
        &self,
        kind: Option<BookKind>,
        per_page: usize,
        page: usize,
        sort_by: BookSortBy,
    ) -> crate::Result<Vec<Book>>;

    async fn get_many_by_ids(&self, book_ids: Vec<u32>) -> crate::Result<Vec<Book>>;

    async fn get_many_by_tags(&self, book_tags: Vec<BookTag>)
        -> crate::Result<Vec<BookGroupByTag>>;

    async fn get_many_by_tag(&self, book_tag: BookTag) -> crate::Result<Vec<Book>>;

    async fn add(&self, book: Book) -> crate::Result<bool>;

    // async fn add_tags(&self, )
}

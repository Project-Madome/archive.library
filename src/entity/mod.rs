mod book;
mod book_tag;

pub use book::*;
pub use book_tag::*;

#[derive(Debug, Clone, Copy)]
pub enum Sort {
    Desc,
    Asc,
}

use serde::Deserialize;

use crate::entity::{self, Sort};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookSortBy {
    IdDesc,
    IdAsc,
    Random,
}

impl From<BookSortBy> for entity::BookSortBy {
    fn from(sort_by: BookSortBy) -> Self {
        use BookSortBy::*;

        match sort_by {
            IdDesc => Self::Id(Sort::Desc),
            IdAsc => Self::Id(Sort::Asc),
            Random => Self::Random,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookKind {
    Doujinshi,
    Manga,
    GameCg,
    ArtistCg,
}

impl From<BookKind> for entity::BookKind {
    fn from(book_kind: BookKind) -> Self {
        use BookKind::*;

        match book_kind {
            Doujinshi => Self::Doujinshi,
            Manga => Self::Manga,
            GameCg => Self::GameCg,
            ArtistCg => Self::ArtistCg,
        }
    }
}

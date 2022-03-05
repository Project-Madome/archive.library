use std::fmt::Display;

use chrono::{DateTime, Utc};

use super::{BookTag, Sort};

#[derive(Debug)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub page: usize,
    pub language: String,
    pub kind: BookKind,
    pub tags: Vec<BookTag>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub enum BookKind {
    Manga,
    Doujinshi,
    ArtistCg,
    GameCg,
}

impl BookKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Manga => "manga",
            Self::Doujinshi => "doujinshi",
            Self::ArtistCg => "artist_cg",
            Self::GameCg => "game_cg",
        }
    }
}

impl From<String> for BookKind {
    fn from(kind: String) -> Self {
        match kind.as_str() {
            "manga" => Self::Manga,
            "doujinshi" => Self::Doujinshi,
            "artist_cg" => Self::ArtistCg,
            "game_cg" => Self::GameCg,
            _ => unreachable!(),
        }
    }
}

pub enum BookSortBy {
    Id(Sort),
    Random,
}

///
/// ```json
/// [
///     [
///         ["female", "anal"]
///         [ ..books ]
///     ]
/// ]
/// ```
///
/// ```json
/// [
///     {
///         tag: ["female", "anal"],
///         books: [
///             ..books
///         ]
///     }
/// ]
/// ```
pub struct BookGroupByTag {
    tag: BookTag,
    books: Vec<Book>,
}

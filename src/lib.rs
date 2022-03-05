mod app;
mod command;
mod config;
mod constant;
mod database;
mod entity;
mod error;
mod model;
mod msg;
mod payload;
mod registry;
mod repository;
mod usecase;

pub use registry::RootRegistry;

use error::Error;

type Result<T> = std::result::Result<T, Error>;

pub fn release() -> bool {
    cfg!(not(debug_assertions))
}

pub fn debug() -> bool {
    cfg!(debug_assertions)
}

pub fn test() -> bool {
    cfg!(test)
}

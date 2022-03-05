use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use util::{body_parser, http::SetResponse};

use crate::{
    config::Config,
    model::Presenter,
    payload,
    usecase::{get_book, get_books, get_books_by_ids},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Msg: {0}")]
    Msg(#[from] MsgError),
    #[error("Command: {0}")]
    Command(#[from] CommandError),
    #[error("UseCase: {0}")]
    UseCase(#[from] UseCaseError),
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Payload: {0}")]
    Payload(#[from] payload::Error),

    #[error("AuthSdk: {0}")]
    AuthSdk(#[from] madome_sdk::api::auth::Error),

    #[error("OldLibrarySdk: {0}")]
    OldLibrarySdk(#[from] madome_sdk::api::old_library::Error),

    // TODO: 나중에 위치 재선정
    #[error("ReadChunksFromBody: {0}")]
    ReadChunksFromBody(#[from] hyper::Error),
}

impl From<body_parser::Error> for Error {
    fn from(err: body_parser::Error) -> Self {
        match err {
            body_parser::Error::JsonDeserialize(e) => payload::Error::JsonDeserialize(e).into(),
            body_parser::Error::NotSupportedContentType(e) => {
                payload::Error::NotSupportedContentType(e).into()
            }
        }
    }
}

type MsgError = crate::msg::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("SeaOrm: {0}")]
    SeaOrm(#[from] sea_orm::DbErr),
}

impl From<sea_orm::DbErr> for crate::Error {
    fn from(error: sea_orm::DbErr) -> Self {
        Error::Repository(error.into())
    }
}

impl From<sea_orm::TransactionError<sea_orm::DbErr>> for crate::Error {
    fn from(err: sea_orm::TransactionError<sea_orm::DbErr>) -> Self {
        match err {
            sea_orm::TransactionError::Connection(err) => Self::Repository(err.into()),
            sea_orm::TransactionError::Transaction(err) => Self::Repository(err.into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {}

#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("GetBooks: {0}")]
    GetBooks(#[from] get_books::Error),

    #[error("GetBook: {0}")]
    GetBook(#[from] get_book::Error),

    #[error("GetBooksByIds: {0}")]
    GetBooksByIds(#[from] get_books_by_ids::Error),

    #[error("CreateBook: ")]
    CreateBook,
}

#[async_trait::async_trait]
impl Presenter for Error {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        resp: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        use crate::msg::Error::*;
        use get_book::Error::*;
        use Error::*;
        use UseCaseError::*;

        match self {
            Msg(NotFound) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body("Not found".into());
            }

            Payload(err) => {
                resp.set_status(StatusCode::BAD_REQUEST).unwrap();
                resp.set_body(err.to_string().into());
            }

            UseCase(GetBook(err @ NotFoundBook)) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body(err.to_string().into());
            }

            AuthSdk(ref err) => {
                use madome_sdk::api::{auth::Error as AuthError, BaseError};

                match err {
                    AuthError::Base(err) => match err {
                        err @ BaseError::Unauthorized => {
                            resp.set_status(StatusCode::UNAUTHORIZED).unwrap();
                            resp.set_body(err.to_string().into());
                        }
                        err @ BaseError::PermissionDenied => {
                            resp.set_status(StatusCode::FORBIDDEN).unwrap();
                            resp.set_body(err.to_string().into());
                        }
                        BaseError::Undefined(code, body) => {
                            resp.set_status(code).unwrap();
                            resp.set_body(body.to_owned().into());
                        }
                        _ => {
                            resp.set_status(StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                            resp.set_body(err.to_string().into());
                        }
                    },
                    _ => {
                        resp.set_status(StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                        resp.set_body(err.to_string().into());
                    }
                }
            }

            err => {
                resp.set_status(StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                resp.set_body(err.to_string().into());
            }
        };

        Ok(())
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),
    #[error("event has not started")]
    EventNotStarted,
    #[error("event has ended")]
    EventEnded,
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Serialize)]
pub struct ErrorResponse<'a> {
    kind: &'a str,
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, kind) = match self {
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            Error::EventNotStarted => (StatusCode::UNAUTHORIZED, "event_not_started"),
            Error::EventEnded => (StatusCode::UNAUTHORIZED, "event_ended"),
        };

        (
            status,
            Json(ErrorResponse {
                kind,
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

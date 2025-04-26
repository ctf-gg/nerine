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
    #[error("{0}")] // TODO(aiden): nit: this is not lowercase, (but it could be)
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("invalid token")]
    InvalidToken,
    #[error("the event has not started")]
    EventNotStarted,
    #[error("the event has ended")]
    EventEnded,
    #[error("wrong flag")]
    WrongFlag,
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
            Error::Jwt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "jwt_error"),
            Error::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            Error::EventNotStarted => (StatusCode::UNAUTHORIZED, "event_not_started"),
            Error::EventEnded => (StatusCode::UNAUTHORIZED, "event_ended"),
            Error::WrongFlag => (StatusCode::BAD_REQUEST, "wrong_flag"),
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

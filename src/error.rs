use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::NaiveDateTime;
use serde::Serialize;
use thiserror::Error;

use crate::EVENT;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),
    #[error("{0}")] // TODO(aiden): nit: this is not lowercase, (but it could be)
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("invalid token")]
    InvalidToken,
    #[error("challenge not found")]
    NotFoundChallenge,
    #[error("team not found")]
    NotFoundTeam,
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

#[derive(Serialize)]
pub struct EventNotStartedResponse<'a> {
    kind: &'a str,
    message: String,
    data: NaiveDateTime,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let (status, kind) = match self {
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            Error::Jwt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "jwt_error"),
            Error::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            Error::NotFoundChallenge | Error::NotFoundTeam => (StatusCode::NOT_FOUND, "not_found"),
            Error::EventNotStarted => {
                // Event not started special cased to return start time
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(EventNotStartedResponse {
                        kind: "event_not_started",
                        message,
                        data: EVENT.start_time,
                    }),
                )
                    .into_response();
            }
            Error::EventEnded => (StatusCode::UNAUTHORIZED, "event_ended"),
            Error::WrongFlag => (StatusCode::BAD_REQUEST, "wrong_flag"),
        };

        (status, Json(ErrorResponse { kind, message })).into_response()
    }
}

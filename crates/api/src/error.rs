use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::NaiveDateTime;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),
    #[error("{}", _0.to_string().to_lowercase())]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("{0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("{0}")]
    Deploy(#[from] reqwest::Error), // TODO this might be used for other classes of error, idk yet
    #[error("Invalid token")]
    InvalidToken,
    #[error("Challenge not found")]
    NotFoundChallenge,
    #[error("Team not found")]
    NotFoundTeam,
    #[error("Division not found")]
    NotFoundDivision,
    #[error("The event has not started, starts at {0}")]
    EventNotStarted(NaiveDateTime),
    #[error("The event has ended")]
    EventEnded,
    #[error("Wrong flag")]
    WrongFlag,
    #[error("Team name already taken")]
    TeamNameTaken,
    #[error(
        "This is a generic error, you shouldn't recieve this is if you're a well behaved client!"
    )]
    GenericError,
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Serialize)]
pub struct ErrorResponse<'a> {
    error: &'a str,
    message: String,
}

#[derive(Serialize)]
pub struct EventNotStartedResponse<'a> {
    error: &'a str,
    message: String,
    data: NaiveDateTime,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let (status, error) = match self {
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            Error::Jwt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "jwt_error"),
            Error::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            Error::Deploy(_) => (StatusCode::BAD_REQUEST, "deploy_error"),
            Error::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            Error::NotFoundChallenge | Error::NotFoundTeam | Error::NotFoundDivision => {
                (StatusCode::NOT_FOUND, "not_found")
            }
            Error::EventNotStarted(start_time) => {
                // Event not started special cased to return start time
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(EventNotStartedResponse {
                        error: "event_not_started",
                        message,
                        data: start_time,
                    }),
                )
                    .into_response();
            }
            Error::EventEnded => (StatusCode::UNAUTHORIZED, "event_ended"),
            Error::WrongFlag => (StatusCode::BAD_REQUEST, "wrong_flag"),
            Error::TeamNameTaken => (StatusCode::BAD_REQUEST, "team_name_taken"),
            Error::GenericError => (StatusCode::BAD_REQUEST, "generic_error"),
        };

        (status, Json(ErrorResponse { error, message })).into_response()
    }
}

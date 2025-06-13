use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use thiserror::Error;
use serde::Serialize;

// TODO(ani): dedupe between api and here
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    JSON(#[from] serde_json::Error),
    #[error("{0}")]
    Docker(#[from] bollard::errors::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("a deployment already exists")]
    AlreadyDeployed,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize)]
pub struct ErrorResponse {
    error: &'static str,
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let (status, error) = match self {
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            Error::JSON(_) => (StatusCode::INTERNAL_SERVER_ERROR, "json_error"),
            Error::Docker(_) => (StatusCode::INTERNAL_SERVER_ERROR, "docker_error"),
            Error::Reqwest(_) => (StatusCode::INTERNAL_SERVER_ERROR, "reqwest_error"),
            Error::AlreadyDeployed => (StatusCode::BAD_REQUEST, "already_deployed"),
        };

        (status, Json(ErrorResponse { error, message })).into_response()
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error: {0}")]
    Database(sqlx::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse<'a> {
    kind: &'a str,
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    kind: "database_error",
                    message: self.to_string(),
                }),
            )
                .into_response(),
        }
    }
}

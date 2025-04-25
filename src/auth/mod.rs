use axum::Router;
use thiserror::Error;

mod register;
mod jwt;

pub fn router() -> Router {
    Router::new()
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("database error")]
    Database,
}
use axum::Router;
use thiserror::Error;

mod account;
mod register;

pub fn router() -> Router {
    Router::new()
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("database error")]
    Database,
}
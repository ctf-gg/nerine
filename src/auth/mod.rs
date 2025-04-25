use axum::{routing::post, Router};
use thiserror::Error;

mod jwt;
mod register;

pub fn router() -> Router {
    Router::new().route("/register", post(register::route))
}

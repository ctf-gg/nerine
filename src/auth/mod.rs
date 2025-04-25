use axum::{routing::post, Router};

mod jwt;
mod register;

pub fn router() -> Router {
    Router::new().route("/register", post(register::route))
}

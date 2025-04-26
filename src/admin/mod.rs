use axum::{routing::{get, post}, Router};

mod challenges;

pub fn router() -> Router {
    Router::new()
}
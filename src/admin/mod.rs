use std::sync::LazyLock;

use axum::{
    extract::FromRequestParts, http::request::Parts, RequestPartsExt, Router
};
mod challenges;

pub fn router() -> Router {
    Router::new().nest("/challs", challenges::router())
}

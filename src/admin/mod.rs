use std::sync::LazyLock;

use axum::{
    extract::FromRequestParts, http::request::Parts, routing::{get, post}, RequestPartsExt, Router
};
use axum_extra::extract::CookieJar;

mod challenges;

pub static ADMIN_TOKEN: LazyLock<String> =
    LazyLock::new(|| std::env::var("ADMIN_TOKEN").expect("expected ADMIN_TOKEN env var"));

#[derive(Debug, Clone)]
pub struct Admin;

impl<S> FromRequestParts<S> for Admin
where
    S: Send + Sync,
{
    type Rejection = sctf::Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let token = jar
            .get("admin_token")
            .ok_or(sctf::Error::InvalidToken)?
            .value();


        if token == ADMIN_TOKEN.as_str() {
            Ok(Admin)
        } else {
            Err(sctf::Error::InvalidToken)
        }
    }
}

pub fn router() -> Router {
    Router::new()
}

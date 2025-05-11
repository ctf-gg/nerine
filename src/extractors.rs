use std::sync::LazyLock;

use crate::jwt::{decode_jwt, Claims};
use axum::{extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;

#[derive(Debug, Clone)]
pub struct Auth(pub Claims);

impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let jwt = jar.get("token").ok_or(crate::Error::InvalidToken)?.value();

        let claims = decode_jwt(jwt)?;

        Ok(Auth(claims))
    }
}

pub static ADMIN_TOKEN: LazyLock<String> =
    LazyLock::new(|| std::env::var("ADMIN_TOKEN").expect("expected ADMIN_TOKEN env var"));

#[derive(Debug, Clone)]
pub struct Admin;

impl<S> FromRequestParts<S> for Admin
where
    S: Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let token = jar
            .get("admin_token")
            .ok_or(crate::Error::InvalidToken)?
            .value();

        if token == ADMIN_TOKEN.as_str() {
            Ok(Admin)
        } else {
            Err(crate::Error::InvalidToken)
        }
    }
}

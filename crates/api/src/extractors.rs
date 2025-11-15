use std::ops::Deref;

use crate::{
    config::Config,
    jwt::{decode_jwt, Claims},
};
use axum::{extract::{FromRequestParts, OptionalFromRequestParts}, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;

#[derive(Debug, Clone)]
pub struct Auth(pub Claims);

impl<S, B> FromRequestParts<B> for Auth
where
    B: Deref<Target = S> + Send + Sync,
    S: AsRef<Config> + Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request_parts(parts: &mut Parts, cfg: &B) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let jwt = jar.get("token").ok_or(crate::Error::InvalidToken)?.value();

        let claims = decode_jwt(&cfg.as_ref().jwt_keys, jwt)?;

        Ok(Auth(claims))
    }
}

impl<S, B> OptionalFromRequestParts<B> for Auth
where
    B: Deref<Target = S> + Send + Sync,
    S: AsRef<Config> + Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request_parts(parts: &mut Parts, cfg: &B) -> Result<Option<Self>, Self::Rejection> {
        match <Auth as FromRequestParts<B>>::from_request_parts(parts, cfg).await {
            Ok(a) => Ok(Some(a)),
            Err(e) => match e {
                crate::Error::InvalidToken => Ok(None),
                _ => Err(e),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Admin;

impl<S, B> FromRequestParts<B> for Admin
where
    B: Deref<Target = S> + Send + Sync,
    S: AsRef<Config> + Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request_parts(parts: &mut Parts, cfg: &B) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let token = jar
            .get("admin_token")
            .ok_or(crate::Error::InvalidToken)?
            .value();

        if token == cfg.as_ref().admin_token {
            Ok(Admin)
        } else {
            Err(crate::Error::InvalidToken)
        }
    }
}

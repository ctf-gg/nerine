use crate::{State, jwt::{decode_jwt, Claims}};
use axum::{extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;

#[derive(Debug, Clone)]
pub struct Auth(pub Claims);

impl FromRequestParts<State> for Auth {
    type Rejection = crate::Error;

    async fn from_request_parts(parts: &mut Parts, cfg: &State) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let jwt = jar.get("token").ok_or(crate::Error::InvalidToken)?.value();

        let claims = decode_jwt(&cfg.jwt_keys, jwt)?;

        Ok(Auth(claims))
    }
}

#[derive(Debug, Clone)]
pub struct Admin;

impl FromRequestParts<State> for Admin {
    type Rejection = crate::Error;

    async fn from_request_parts(parts: &mut Parts, cfg: &State) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let token = jar
            .get("admin_token")
            .ok_or(crate::Error::InvalidToken)?
            .value();

        if token == &cfg.admin_token {
            Ok(Admin)
        } else {
            Err(crate::Error::InvalidToken)
        }
    }
}

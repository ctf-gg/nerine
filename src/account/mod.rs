use axum::{
    extract::FromRequestParts, http::request::Parts, routing::post, RequestPartsExt, Router,
};
use axum_extra::extract::CookieJar;
use jwt::Claims;

mod jwt;
mod routes;

#[derive(Debug, Clone)]
pub struct Auth(pub Claims);

impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = sctf::Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await.unwrap(); //infailable
        let jwt = jar.get("token").ok_or(sctf::Error::InvalidToken)?.value();

        let claims = jwt::decode_jwt(jwt)?;

        Ok(Auth(claims))
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/register", post(routes::register))
        .route("/login", post(routes::login))
        .route("/update_profile", post(routes::update_profile))
}
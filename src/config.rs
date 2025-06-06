use std::{str::FromStr, sync::Arc};

use envconfig::Envconfig;
use jsonwebtoken::{DecodingKey, EncodingKey};

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl FromStr for JwtKeys {
    type Err = jsonwebtoken::errors::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            encoding: EncodingKey::from_base64_secret(s)?,
            decoding: DecodingKey::from_base64_secret(s)?,
        })
    }
}

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,

    #[envconfig(from = "JWT_SECRET")]
    pub jwt_keys: JwtKeys,

    #[envconfig(from = "ADMIN_TOKEN")]
    pub admin_token: String,
}

/* subject to change */
pub type State = Arc<Config>;

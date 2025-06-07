use std::{str::FromStr, sync::Arc};

use envconfig::Envconfig;
use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::{event::Event, DB};

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

    #[envconfig(from = "EVENT_PATH", default = "event.toml")]
    pub event_path: String,
}

pub struct StateInner {
    pub config: Config,
    pub event: Event,
    pub db: DB,
}

impl AsRef<Config> for StateInner {
    fn as_ref(&self) -> &Config {
        &self.config
    }
}

impl AsRef<DB> for StateInner {
    fn as_ref(&self) -> &DB {
        &self.db
    }
}

/* subject to change */
pub type State = Arc<StateInner>;

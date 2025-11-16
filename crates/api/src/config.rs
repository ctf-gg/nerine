use std::{str::FromStr, sync::Arc};

use envconfig::Envconfig;
use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::{email, event::Event, DB};

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

    #[envconfig(from = "CORS_ORIGIN", default = "http://nerine.localhost")]
    pub cors_origin: String,

    #[envconfig(from = "SMTP_URL", default = "")]
    pub smtp_url: String,

    #[envconfig(from = "FROM_EMAIL", default = "noreply@nerine.localhost")]
    pub from_email: String,

    #[envconfig(from = "DEPLOYER_BASE", default = "http://deployer:3001")]
    pub deployer_base: String,

    #[envconfig(from = "BLOODBOT_DISCORD_WEBHOOK_URL")]
    pub bloodbot_discord_webhook_url: Option<String>,
}

pub struct StateInner {
    pub config: Config,
    pub event: Event,
    pub db: DB,
    pub email: email::EmailService,
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

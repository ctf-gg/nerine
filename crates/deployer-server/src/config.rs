use std::{collections::HashMap, sync::Arc};

use deployer_common::challenge::Challenge;
use envconfig::Envconfig;
use sqlx::PgPool;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
}

pub struct StateInner {
    pub config: Config,
    // keyed by id
    pub challenge_data: HashMap<String, Challenge>,
    pub db: PgPool,
}

pub type State = Arc<StateInner>;

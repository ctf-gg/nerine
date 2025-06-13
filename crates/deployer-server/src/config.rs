use std::{collections::HashMap, ops::Deref, path::{Path, PathBuf}, str::FromStr, sync::Arc};

use deployer_common::challenge::{Challenge, DeployableContextData};
use envconfig::Envconfig;
use log::debug;
use serde::Deserialize;
use sqlx::PgPool;
use eyre::eyre;

// god-awful keychain-type thing
#[derive(Debug, Clone, Deserialize)]
pub struct HostKeychain {
    // host id ("default" is fallback)
    pub id: String,
    // docker stuff
    pub docker: DeployableContextData,
    // caddy stuff
    pub caddy: CaddyKeychain,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CaddyKeychain {
    // endpoint
    pub endpoint: String,
    // base subdomain
    // subdomains of the form <subdomain>.<base>
    pub base: String,
    #[serde(flatten)]
    pub mtls: ClientTLSKeychain,
}

impl CaddyKeychain {
    pub fn as_client(&self) -> crate::Result<reqwest::Client> {
        Ok(reqwest::ClientBuilder::new()
            .tls_built_in_root_certs(false)
            // FIXME(ani): currently not verifying against ca certs because caddy sucks
            .add_root_certificate(reqwest::Certificate::from_pem(self.mtls.cacert.as_bytes())?)
            .danger_accept_invalid_hostnames(true)
            .identity(reqwest::Identity::from_pem(format!("{}\n{}", self.mtls.key, self.mtls.cert).as_bytes())?)
            .build()?)
    }

    pub fn prep_url(&self, path: &str) -> reqwest::Url {
        // unwrap bad
        reqwest::Url::parse(&self.endpoint).unwrap().join(path).unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientTLSKeychain {
    // ca cert (pem)
    pub cacert: String,
    // client cert (pem)
    pub cert: String,
    // client key (pem)
    pub key: String,
}

pub struct HostKeychainEnv(HashMap<String, HostKeychain>);

//#[derive(Debug, Error)]
//pub enum HostKeychainEnvError {
//    #[error("duplicate host keychain: {0}")]
//    DuplicateKey(String),
//    #[error("missing default host keychain")]
//    MissingDefault,
//    #[error("{0}")]
//    Json(#[from] serde_json::Error),
//}

impl FromStr for HostKeychainEnv {
    type Err = eyre::Error;

    fn from_str(s: &str) -> eyre::Result<Self> {
        use base64::prelude::*;

        let decoded = BASE64_STANDARD.decode(s)?;
        //debug!("decoded = {}", std::str::from_utf8(&decoded).unwrap());
        let parsed = serde_json::from_slice::<Vec<HostKeychain>>(&decoded)?;
        //debug!("parsed = {:?}", parsed);
        let mut m = HashMap::new();
        for chain in parsed {
            if m.contains_key(&chain.id) {
                //return Err(HostKeychainEnvError::DuplicateKey(chain.id));
                return Err(eyre!("duplicate key {}", chain.id));
            }
            m.insert(chain.id.clone(), chain);
        }
        if !m.contains_key("default") {
            //Err(HostKeychainEnvError::MissingDefault)
            Err(eyre!("missing default host"))
        } else {
            Ok(Self(m))
        }
    }
}

impl Deref for HostKeychainEnv {
    type Target = HashMap<String, HostKeychain>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
    // expected as base64
    #[envconfig(from = "HOST_KEYCHAINS")]
    pub host_keychains: HostKeychainEnv,
    #[envconfig(from = "CHALLENGES_DIR")]
    pub challenges_dir: PathBuf,
}

pub fn load_challenges_from_dir(dir: &Path) -> eyre::Result<HashMap<String, Challenge>> {
    let mut m = HashMap::new();
    for pat in glob::glob(dir.join("*.toml").to_str().ok_or_else(|| eyre!("bad string for pattern"))?)? {
        if let Ok(pat) = pat {
            let chall_s = std::fs::read_to_string(pat)?;
            let chall = toml::from_str::<Challenge>(&chall_s)?;
            if m.contains_key(&chall.id) {
                return Err(eyre!("Duplicate challenge {}", chall.id));
            }
            m.insert(chall.id.clone(), chall);
        }
    }
    Ok(m)
}

pub struct StateInner {
    pub config: Config,
    // keyed by id
    pub challenge_data: HashMap<String, Challenge>,
    pub db: PgPool,
}

pub type State = Arc<StateInner>;

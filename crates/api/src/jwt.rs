use jsonwebtoken::errors::Result;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub exp: u64,
    pub team_id: String,
}

pub fn generate_jwt(
    keys: &config::JwtKeys,
    team_id: &str,
    exp: chrono::Duration,
) -> Result<String> {
    jsonwebtoken::encode(
        &Default::default(),
        &Claims {
            team_id: team_id.to_string(),
            exp: jsonwebtoken::get_current_timestamp() + exp.num_seconds() as u64,
        },
        &keys.encoding,
    )
}

pub fn decode_jwt(keys: &config::JwtKeys, jwt: &str) -> crate::Result<Claims> {
    // TODO(aiden): this logic is pretty ugly, but there doesn't seem to be a good way to handle it
    // since the constructor for the error in jsonwebtoken is private
    // keep trying i guess though.
    Ok(
        jsonwebtoken::decode::<Claims>(jwt, &keys.decoding, &Default::default())
            .map_err(|e| {
                if *e.kind() == jsonwebtoken::errors::ErrorKind::InvalidToken {
                    crate::Error::InvalidToken
                } else {
                    e.into()
                }
            })?
            .claims,
    )
}

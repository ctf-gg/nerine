use jsonwebtoken::{errors::Result, DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub exp: u64,
    pub team_id: String,
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys {
        encoding: EncodingKey::from_base64_secret(&secret).expect("JWT_SECRET is not valid base64"),
        decoding: DecodingKey::from_base64_secret(&secret).expect("JWT_SECRET is not valid base64"),
    }
});

pub fn generate_jwt(team_id: &str, exp: chrono::Duration) -> Result<String> {
    jsonwebtoken::encode(
        &Default::default(),
        &Claims {
            team_id: team_id.to_string(),
            exp: jsonwebtoken::get_current_timestamp() + exp.num_seconds() as u64,
        },
        &KEYS.encoding,
    )
}

pub fn decode_jwt(jwt: &str) -> crate::Result<Claims> {
    // TODO(aiden): this logic is pretty ugly, but there doesn't seem to be a good way to handle it
    // since the constructor for the error in jsonwebtoken is private
    // keep trying i guess though.
    Ok(
        jsonwebtoken::decode::<Claims>(jwt, &KEYS.decoding, &Default::default())
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

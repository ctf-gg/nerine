use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
    team_id: String,
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

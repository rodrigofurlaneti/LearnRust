use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::application::token_service::{AccessToken, TokenService};
use crate::domain::UserId;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

/// Adapter concreto do port `TokenService`, usando JWT assinado com HMAC.
pub struct JwtTokenService {
    secret: String,
    ttl_seconds: u64,
}

impl JwtTokenService {
    pub fn new(secret: String, ttl_seconds: u64) -> Self {
        Self { secret, ttl_seconds }
    }

    fn expiration_timestamp(&self) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("relógio do sistema inválido")
            .as_secs();

        (now + self.ttl_seconds) as usize
    }
}

impl TokenService for JwtTokenService {
    fn issue(&self, user_id: UserId) -> Result<AccessToken, String> {
        let claims = Claims {
            sub: user_id.as_uuid().to_string(),
            exp: self.expiration_timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map(AccessToken)
        .map_err(|error| error.to_string())
    }
}

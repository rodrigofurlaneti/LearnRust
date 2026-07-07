use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::shared::token_service::{AccessToken, TokenClaims, TokenService};
use crate::domain::{Role, UserId};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

/// Adapter concreto do port `TokenService`, usando JWT assinado com HMAC.
/// Herdado sem alteracoes da Semana 9.
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
            .expect("relogio do sistema invalido")
            .as_secs();

        (now + self.ttl_seconds) as usize
    }
}

impl TokenService for JwtTokenService {
    fn issue(&self, user_id: UserId, role: Role) -> Result<AccessToken, String> {
        let claims = Claims {
            sub: user_id.as_uuid().to_string(),
            role: role.as_str().to_string(),
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

    fn verify(&self, token: &str) -> Result<TokenClaims, String> {
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|error| error.to_string())?;

        let user_id = Uuid::parse_str(&decoded.claims.sub).map_err(|error| error.to_string())?;
        let role = Role::parse(&decoded.claims.role)
            .ok_or_else(|| "role desconhecida no token".to_string())?;

        Ok(TokenClaims {
            user_id: UserId::from_uuid(user_id),
            role,
        })
    }
}

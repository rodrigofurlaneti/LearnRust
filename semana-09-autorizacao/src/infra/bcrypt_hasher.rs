use async_trait::async_trait;

use crate::domain::{HashedPassword, PasswordHasher, PlainPassword};

/// Adapter concreto do port `PasswordHasher`, usando bcrypt.
/// `cost` fica configurável para poder baixar em testes e subir em produção.
pub struct BcryptPasswordHasher {
    cost: u32,
}

impl BcryptPasswordHasher {
    pub fn new(cost: u32) -> Self {
        Self { cost }
    }
}

#[async_trait]
impl PasswordHasher for BcryptPasswordHasher {
    async fn hash(&self, password: &PlainPassword) -> HashedPassword {
        let cost = self.cost;
        let raw = password.expose().to_string();

        let hash = tokio::task::spawn_blocking(move || {
            bcrypt::hash(raw, cost).expect("falha ao gerar hash bcrypt")
        })
        .await
        .expect("thread de hashing falhou");

        HashedPassword::from_hash(hash)
    }

    async fn verify(&self, password: &PlainPassword, hashed: &HashedPassword) -> bool {
        let raw = password.expose().to_string();
        let hash = hashed.as_str().to_string();

        tokio::task::spawn_blocking(move || bcrypt::verify(raw, &hash).unwrap_or(false))
            .await
            .unwrap_or(false)
    }
}

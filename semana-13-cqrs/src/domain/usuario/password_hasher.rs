use async_trait::async_trait;

use crate::domain::usuario::password::{HashedPassword, PlainPassword};

/// Porta (Dependency Inversion Principle): o dominio declara o contrato de
/// hashing/verificacao de senha, mas nao sabe nada sobre bcrypt, argon2 etc.
/// A implementacao concreta mora na camada de infraestrutura.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &PlainPassword) -> HashedPassword;
    async fn verify(&self, password: &PlainPassword, hashed: &HashedPassword) -> bool;
}

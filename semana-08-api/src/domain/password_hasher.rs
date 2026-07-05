use async_trait::async_trait;

use crate::domain::password::{HashedPassword, PlainPassword};

/// Porta (Dependency Inversion Principle): o domínio declara o contrato de
/// hashing/verificação de senha, mas não sabe nada sobre bcrypt, argon2 etc.
/// A implementação concreta mora na camada de infraestrutura.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &PlainPassword) -> HashedPassword;
    async fn verify(&self, password: &PlainPassword, hashed: &HashedPassword) -> bool;
}

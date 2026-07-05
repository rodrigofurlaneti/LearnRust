use async_trait::async_trait;

use crate::domain::email::Email;
use crate::domain::errors::DomainError;
use crate::domain::user::User;
use crate::domain::user_id::UserId;

/// Porta de saida (Dependency Inversion Principle): o dominio e a aplicacao
/// dependem desta abstracao, nunca de Postgres/SQLx diretamente. Quem
/// implementa e a camada de infraestrutura.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;

    // Ainda nao usado pelos casos de uso atuais (registro/login), mas faz
    // parte do contrato do agregado - ex.: um futuro caso de uso "GET /me".
    #[allow(dead_code)]
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError>;
}

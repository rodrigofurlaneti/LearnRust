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

    /// Usado pela rota `GET /me` (extractor `AuthenticatedUser` decodifica
    /// o `UserId` do JWT, o caso de uso busca os dados atuais aqui).
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError>;
}

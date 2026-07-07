use async_trait::async_trait;

use crate::domain::email::Email;
use crate::domain::errors::DomainError;
use crate::domain::user::User;
use crate::domain::user_id::UserId;

/// Porta de saida (Dependency Inversion Principle): o dominio e a aplicacao
/// dependem desta abstracao, nunca da implementacao concreta de
/// persistencia. Na Semana 10 quem implementa e um cache em memoria
/// (`infra::cache_user_repository`) decorado com auditoria
/// (`infra::audited_user_repository`) - nenhuma das duas coisas aparece
/// aqui, o dominio continua sem saber como/onde os dados moram.
///
/// `update` e `delete` sao novos nesta semana: a Semana 9 so tinha
/// `save`/`find_by_*` porque o unico jeito de "escrever" um usuario era o
/// cadastro publico. Agora existe gestao administrativa de usuarios
/// (`PUT`/`DELETE /admin/usuarios/:id`), entao o contrato precisa cobrir
/// os tres tipos de escrita que o log de auditoria precisa distinguir:
/// insert, update e delete.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn update(&self, user: &User) -> Result<(), DomainError>;
    async fn delete(&self, id: UserId) -> Result<(), DomainError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;

    /// Usado pela rota `GET /me` (extractor `AuthenticatedUser` decodifica
    /// o `UserId` do JWT, o caso de uso busca os dados atuais aqui).
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError>;
}

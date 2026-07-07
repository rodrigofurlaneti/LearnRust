use std::sync::Arc;

use async_trait::async_trait;

use crate::application::audit_logger::{AuditAction, AuditEntity, AuditEntry, AuditLogger};
use crate::domain::{DomainError, Email, User, UserId, UserRepository};

/// Decorator (padrao GoF, aplicado aqui como reforco pratico de
/// Open/Closed Principle): implementa o MESMO port `UserRepository` que
/// `CacheUserRepository`, mas acrescenta uma linha de auditoria a cada
/// escrita, sem que `CacheUserRepository` ou os casos de uso
/// (`RegisterUser`/`UpdateUser`/`DeleteUser`) precisem saber que isso
/// acontece. E o UNICO lugar do sistema que conhece as duas pontas do
/// requisito da Semana 10: "grave no cache" e "grave no historico".
///
/// So decora escritas (`save`/`update`/`delete`) - leituras (`find_by_*`)
/// so delegam direto para o repositorio interno, porque o pedido da
/// semana e auditar insert/update/delete, nao consultas.
pub struct AuditedUserRepository {
    inner: Arc<dyn UserRepository>,
    audit: Arc<dyn AuditLogger>,
}

impl AuditedUserRepository {
    pub fn new(inner: Arc<dyn UserRepository>, audit: Arc<dyn AuditLogger>) -> Self {
        Self { inner, audit }
    }

    fn entry_for(user: &User, action: AuditAction) -> AuditEntry {
        AuditEntry::new(
            AuditEntity::Usuario,
            action,
            user.id().as_uuid().to_string(),
            format!("email={} role={}", user.email().as_str(), user.role().as_str()),
        )
    }

    fn deletion_entry_for(id: UserId) -> AuditEntry {
        AuditEntry::new(
            AuditEntity::Usuario,
            AuditAction::Delete,
            id.as_uuid().to_string(),
            "usuario removido do cache".to_string(),
        )
    }
}

#[async_trait]
impl UserRepository for AuditedUserRepository {
    async fn save(&self, user: &User) -> Result<(), DomainError> {
        self.inner.save(user).await?;
        self.audit.record(Self::entry_for(user, AuditAction::Insert));
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        self.inner.update(user).await?;
        self.audit.record(Self::entry_for(user, AuditAction::Update));
        Ok(())
    }

    async fn delete(&self, id: UserId) -> Result<(), DomainError> {
        self.inner.delete(id).await?;
        self.audit.record(Self::deletion_entry_for(id));
        Ok(())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        self.inner.find_by_email(email).await
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        self.inner.find_by_id(id).await
    }
}

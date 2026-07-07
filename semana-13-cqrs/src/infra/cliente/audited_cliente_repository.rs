use std::sync::Arc;

use async_trait::async_trait;

use crate::application::shared::audit_logger::{AuditAction, AuditEntity, AuditEntry, AuditLogger, AuditPayload};
use crate::domain::{Cliente, ClienteId, ClienteRepository, Cpf, DomainError};

/// Decorator simetrico a `AuditedUserRepository`, agora para o agregado
/// `Cliente`. A duplicacao estrutural entre os decorators e intencional e
/// pequena o suficiente para nao justificar uma generalizacao via
/// generics/trait extra neste momento (ver README) - preferimos manter
/// cada decorator lendo como um caso simples e independente a introduzir
/// uma abstracao prematura.
pub struct AuditedClienteRepository {
    inner: Arc<dyn ClienteRepository>,
    audit: Arc<dyn AuditLogger>,
}

impl AuditedClienteRepository {
    pub fn new(inner: Arc<dyn ClienteRepository>, audit: Arc<dyn AuditLogger>) -> Self {
        Self { inner, audit }
    }

    fn entry_for(cliente: &Cliente, action: AuditAction) -> AuditEntry {
        let payload = AuditPayload::Cliente {
            nome: cliente.nome().as_str().to_string(),
            documento: cliente.documento().as_str().to_string(),
            email: cliente.email().as_str().to_string(),
        };
        AuditEntry::new(AuditEntity::Cliente, action, cliente.id().as_uuid().to_string(), payload)
    }

    fn deletion_entry_for(id: ClienteId) -> AuditEntry {
        AuditEntry::new(
            AuditEntity::Cliente,
            AuditAction::Delete,
            id.as_uuid().to_string(),
            AuditPayload::Removido,
        )
    }
}

#[async_trait]
impl ClienteRepository for AuditedClienteRepository {
    async fn save(&self, cliente: &Cliente) -> Result<(), DomainError> {
        self.inner.save(cliente).await?;
        self.audit.record(Self::entry_for(cliente, AuditAction::Insert));
        Ok(())
    }

    async fn update(&self, cliente: &Cliente) -> Result<(), DomainError> {
        self.inner.update(cliente).await?;
        self.audit.record(Self::entry_for(cliente, AuditAction::Update));
        Ok(())
    }

    async fn delete(&self, id: ClienteId) -> Result<(), DomainError> {
        self.inner.delete(id).await?;
        self.audit.record(Self::deletion_entry_for(id));
        Ok(())
    }

    async fn find_by_id(&self, id: ClienteId) -> Result<Option<Cliente>, DomainError> {
        self.inner.find_by_id(id).await
    }

    async fn find_by_documento(&self, documento: &Cpf) -> Result<Option<Cliente>, DomainError> {
        self.inner.find_by_documento(documento).await
    }

    async fn list_all(&self) -> Result<Vec<Cliente>, DomainError> {
        self.inner.list_all().await
    }
}

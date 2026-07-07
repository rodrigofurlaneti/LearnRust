use std::sync::Arc;

use async_trait::async_trait;

use crate::application::audit_logger::{AuditAction, AuditEntity, AuditEntry, AuditLogger, AuditPayload};
use crate::domain::{DomainError, Produto, ProdutoId, ProdutoRepository};

/// Decorator simetrico a `AuditedClienteRepository`, agora para o
/// agregado `Produto`.
pub struct AuditedProdutoRepository {
    inner: Arc<dyn ProdutoRepository>,
    audit: Arc<dyn AuditLogger>,
}

impl AuditedProdutoRepository {
    pub fn new(inner: Arc<dyn ProdutoRepository>, audit: Arc<dyn AuditLogger>) -> Self {
        Self { inner, audit }
    }

    fn entry_for(produto: &Produto, action: AuditAction) -> AuditEntry {
        let payload = AuditPayload::Produto {
            nome: produto.nome().as_str().to_string(),
            preco_centavos: produto.preco().as_centavos(),
        };
        AuditEntry::new(AuditEntity::Produto, action, produto.id().as_uuid().to_string(), payload)
    }

    fn deletion_entry_for(id: ProdutoId) -> AuditEntry {
        AuditEntry::new(
            AuditEntity::Produto,
            AuditAction::Delete,
            id.as_uuid().to_string(),
            AuditPayload::Removido,
        )
    }
}

#[async_trait]
impl ProdutoRepository for AuditedProdutoRepository {
    async fn save(&self, produto: &Produto) -> Result<(), DomainError> {
        self.inner.save(produto).await?;
        self.audit.record(Self::entry_for(produto, AuditAction::Insert));
        Ok(())
    }

    async fn update(&self, produto: &Produto) -> Result<(), DomainError> {
        self.inner.update(produto).await?;
        self.audit.record(Self::entry_for(produto, AuditAction::Update));
        Ok(())
    }

    async fn delete(&self, id: ProdutoId) -> Result<(), DomainError> {
        self.inner.delete(id).await?;
        self.audit.record(Self::deletion_entry_for(id));
        Ok(())
    }

    async fn find_by_id(&self, id: ProdutoId) -> Result<Option<Produto>, DomainError> {
        self.inner.find_by_id(id).await
    }

    async fn list_all(&self) -> Result<Vec<Produto>, DomainError> {
        self.inner.list_all().await
    }
}

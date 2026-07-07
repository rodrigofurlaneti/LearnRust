use std::sync::Arc;

use async_trait::async_trait;

use crate::application::shared::audit_logger::{
    AuditAction, AuditEntity, AuditEntry, AuditLogger, AuditPayload, ItemPedidoSnapshot,
};
use crate::domain::{DomainError, ItemPedido, Pedido, PedidoId, PedidoRepository};

/// Decorator simetrico aos demais, agora para o agregado `Pedido`. Sem
/// `update` porque a porta `PedidoRepository` tambem nao tem (ver
/// `domain::pedido_repository`).
pub struct AuditedPedidoRepository {
    inner: Arc<dyn PedidoRepository>,
    audit: Arc<dyn AuditLogger>,
}

impl AuditedPedidoRepository {
    pub fn new(inner: Arc<dyn PedidoRepository>, audit: Arc<dyn AuditLogger>) -> Self {
        Self { inner, audit }
    }

    fn entry_for(pedido: &Pedido, action: AuditAction) -> AuditEntry {
        let itens = pedido.itens().iter().map(Self::item_to_snapshot).collect();
        let payload = AuditPayload::Pedido {
            cliente_id: pedido.cliente_id().as_uuid().to_string(),
            itens,
            valor_total_centavos: pedido.valor_total().as_centavos(),
        };
        AuditEntry::new(AuditEntity::Pedido, action, pedido.id().as_uuid().to_string(), payload)
    }

    fn item_to_snapshot(item: &ItemPedido) -> ItemPedidoSnapshot {
        ItemPedidoSnapshot {
            produto_id: item.produto_id().as_uuid().to_string(),
            nome_produto: item.nome_produto().as_str().to_string(),
            quantidade: item.quantidade().valor(),
            valor_unitario_centavos: item.valor_unitario().as_centavos(),
            valor_total_centavos: item.valor_total().as_centavos(),
        }
    }

    fn deletion_entry_for(id: PedidoId) -> AuditEntry {
        AuditEntry::new(
            AuditEntity::Pedido,
            AuditAction::Delete,
            id.as_uuid().to_string(),
            AuditPayload::Removido,
        )
    }
}

#[async_trait]
impl PedidoRepository for AuditedPedidoRepository {
    async fn save(&self, pedido: &Pedido) -> Result<(), DomainError> {
        self.inner.save(pedido).await?;
        self.audit.record(Self::entry_for(pedido, AuditAction::Insert));
        Ok(())
    }

    async fn delete(&self, id: PedidoId) -> Result<(), DomainError> {
        self.inner.delete(id).await?;
        self.audit.record(Self::deletion_entry_for(id));
        Ok(())
    }

    async fn find_by_id(&self, id: PedidoId) -> Result<Option<Pedido>, DomainError> {
        self.inner.find_by_id(id).await
    }

    async fn list_all(&self) -> Result<Vec<Pedido>, DomainError> {
        self.inner.list_all().await
    }
}

use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::domain::{DomainError, PedidoId, PedidoRepository};

/// Caso de uso de cancelamento de pedido (`DELETE /pedidos/:id`). Nao
/// existe `UpdateProduto` para pedido de proposito (ver
/// `domain::pedido_repository`) - um pedido errado e cancelado e refeito.
pub struct DeletePedido {
    repository: Arc<dyn PedidoRepository>,
}

impl DeletePedido {
    pub fn new(repository: Arc<dyn PedidoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, pedido_id: PedidoId) -> Result<(), ApplicationError> {
        self.ensure_pedido_existe(pedido_id).await?;
        self.repository.delete(pedido_id).await?;
        Ok(())
    }

    async fn ensure_pedido_existe(&self, pedido_id: PedidoId) -> Result<(), ApplicationError> {
        let existente = self.repository.find_by_id(pedido_id).await?;
        existente
            .map(|_| ())
            .ok_or(ApplicationError::Domain(DomainError::PedidoNotFound))
    }
}

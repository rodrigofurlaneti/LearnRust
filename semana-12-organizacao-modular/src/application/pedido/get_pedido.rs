use std::sync::Arc;

use crate::application::shared::dto::PedidoOutput;
use crate::application::shared::errors::ApplicationError;
use crate::application::pedido::register_pedido::RegisterPedido;
use crate::domain::{DomainError, PedidoId, PedidoRepository};

/// Caso de uso de consulta de um pedido pelo id (`GET /pedidos/:id`).
pub struct GetPedido {
    repository: Arc<dyn PedidoRepository>,
}

impl GetPedido {
    pub fn new(repository: Arc<dyn PedidoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, pedido_id: PedidoId) -> Result<PedidoOutput, ApplicationError> {
        let pedido = self
            .repository
            .find_by_id(pedido_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::PedidoNotFound))?;

        Ok(RegisterPedido::to_output(&pedido))
    }
}

use std::sync::Arc;

use crate::application::dto::ListPedidosOutput;
use crate::application::errors::ApplicationError;
use crate::application::register_pedido::RegisterPedido;
use crate::domain::PedidoRepository;

/// Caso de uso de listagem de pedidos (`GET /pedidos`).
pub struct ListPedidos {
    repository: Arc<dyn PedidoRepository>,
}

impl ListPedidos {
    pub fn new(repository: Arc<dyn PedidoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<ListPedidosOutput, ApplicationError> {
        let pedidos = self.repository.list_all().await?;
        let output = pedidos.iter().map(RegisterPedido::to_output).collect();

        Ok(ListPedidosOutput { pedidos: output })
    }
}

//! Queries do agregado Pedido.

use async_trait::async_trait;

use crate::application::pedido::get_pedido::GetPedido;
use crate::application::pedido::list_pedidos::ListPedidos;
use crate::application::shared::cqrs::{Query, QueryHandler};
use crate::application::shared::dto::{ListPedidosOutput, PedidoOutput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::PedidoId;

/// Pergunta pelos dados de um pedido especifico.
pub struct GetPedidoQuery {
    pub pedido_id: PedidoId,
}

impl Query for GetPedidoQuery {
    type Output = PedidoOutput;
}

#[async_trait]
impl QueryHandler<GetPedidoQuery> for GetPedido {
    async fn handle(&self, query: GetPedidoQuery) -> Result<PedidoOutput, ApplicationError> {
        self.execute(query.pedido_id).await
    }
}

/// Pergunta pela lista completa de pedidos cadastrados.
pub struct ListPedidosQuery;

impl Query for ListPedidosQuery {
    type Output = ListPedidosOutput;
}

#[async_trait]
impl QueryHandler<ListPedidosQuery> for ListPedidos {
    async fn handle(&self, _query: ListPedidosQuery) -> Result<ListPedidosOutput, ApplicationError> {
        self.execute().await
    }
}

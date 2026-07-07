//! Queries do agregado Cliente.

use async_trait::async_trait;

use crate::application::cliente::get_cliente::GetCliente;
use crate::application::cliente::list_clientes::ListClientes;
use crate::application::shared::cqrs::{Query, QueryHandler};
use crate::application::shared::dto::{ClienteOutput, ListClientesOutput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::ClienteId;

/// Pergunta pelos dados de um cliente especifico.
pub struct GetClienteQuery {
    pub cliente_id: ClienteId,
}

impl Query for GetClienteQuery {
    type Output = ClienteOutput;
}

#[async_trait]
impl QueryHandler<GetClienteQuery> for GetCliente {
    async fn handle(&self, query: GetClienteQuery) -> Result<ClienteOutput, ApplicationError> {
        self.execute(query.cliente_id).await
    }
}

/// Pergunta pela lista completa de clientes cadastrados. Sem campos - a
/// consulta nao tem parametros, so a intencao "liste todos" (paginacao e
/// filtros ficam para uma semana futura, ver README).
pub struct ListClientesQuery;

impl Query for ListClientesQuery {
    type Output = ListClientesOutput;
}

#[async_trait]
impl QueryHandler<ListClientesQuery> for ListClientes {
    async fn handle(&self, _query: ListClientesQuery) -> Result<ListClientesOutput, ApplicationError> {
        self.execute().await
    }
}

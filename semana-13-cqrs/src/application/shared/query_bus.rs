//! `QueryBus`: unico ponto de entrada da presentation para qualquer
//! operacao de leitura. Mesma ideia do `CommandBus` (ver comentario la
//! para o racional de resolver os handlers em tempo de compilacao em vez
//! de via `TypeId`/`dyn Any`).

use std::sync::Arc;

use async_trait::async_trait;

use crate::application::cliente::get_cliente::GetCliente;
use crate::application::cliente::list_clientes::ListClientes;
use crate::application::cliente::queries::{GetClienteQuery, ListClientesQuery};
use crate::application::pedido::get_pedido::GetPedido;
use crate::application::pedido::list_pedidos::ListPedidos;
use crate::application::pedido::queries::{GetPedidoQuery, ListPedidosQuery};
use crate::application::produto::get_produto::GetProduto;
use crate::application::produto::list_produtos::ListProdutos;
use crate::application::produto::queries::{GetProdutoQuery, ListProdutosQuery};
use crate::application::shared::dto::{
    ClienteOutput, CurrentUserOutput, ListClientesOutput, ListPedidosOutput, ListProdutosOutput,
    PedidoOutput, ProdutoOutput,
};
use crate::application::shared::cqrs::{Query, QueryHandler};
use crate::application::shared::errors::ApplicationError;
use crate::application::usuario::get_current_user::GetCurrentUser;
use crate::application::usuario::queries::GetCurrentUserQuery;

pub struct QueryBus {
    get_current_user: Arc<GetCurrentUser>,
    get_cliente: Arc<GetCliente>,
    list_clientes: Arc<ListClientes>,
    get_produto: Arc<GetProduto>,
    list_produtos: Arc<ListProdutos>,
    get_pedido: Arc<GetPedido>,
    list_pedidos: Arc<ListPedidos>,
}

impl QueryBus {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        get_current_user: Arc<GetCurrentUser>,
        get_cliente: Arc<GetCliente>,
        list_clientes: Arc<ListClientes>,
        get_produto: Arc<GetProduto>,
        list_produtos: Arc<ListProdutos>,
        get_pedido: Arc<GetPedido>,
        list_pedidos: Arc<ListPedidos>,
    ) -> Self {
        Self {
            get_current_user,
            get_cliente,
            list_clientes,
            get_produto,
            list_produtos,
            get_pedido,
            list_pedidos,
        }
    }

    /// Porta de entrada unica usada pela presentation: `bus.dispatch(SomeQuery{...}).await`.
    pub async fn dispatch<Q>(&self, query: Q) -> Result<Q::Output, ApplicationError>
    where
        Q: Query,
        Self: QueryHandler<Q>,
    {
        QueryHandler::handle(self, query).await
    }
}

#[async_trait]
impl QueryHandler<GetCurrentUserQuery> for QueryBus {
    async fn handle(&self, query: GetCurrentUserQuery) -> Result<CurrentUserOutput, ApplicationError> {
        self.get_current_user.handle(query).await
    }
}

#[async_trait]
impl QueryHandler<GetClienteQuery> for QueryBus {
    async fn handle(&self, query: GetClienteQuery) -> Result<ClienteOutput, ApplicationError> {
        self.get_cliente.handle(query).await
    }
}

#[async_trait]
impl QueryHandler<ListClientesQuery> for QueryBus {
    async fn handle(&self, query: ListClientesQuery) -> Result<ListClientesOutput, ApplicationError> {
        self.list_clientes.handle(query).await
    }
}

#[async_trait]
impl QueryHandler<GetProdutoQuery> for QueryBus {
    async fn handle(&self, query: GetProdutoQuery) -> Result<ProdutoOutput, ApplicationError> {
        self.get_produto.handle(query).await
    }
}

#[async_trait]
impl QueryHandler<ListProdutosQuery> for QueryBus {
    async fn handle(&self, query: ListProdutosQuery) -> Result<ListProdutosOutput, ApplicationError> {
        self.list_produtos.handle(query).await
    }
}

#[async_trait]
impl QueryHandler<GetPedidoQuery> for QueryBus {
    async fn handle(&self, query: GetPedidoQuery) -> Result<PedidoOutput, ApplicationError> {
        self.get_pedido.handle(query).await
    }
}

#[async_trait]
impl QueryHandler<ListPedidosQuery> for QueryBus {
    async fn handle(&self, query: ListPedidosQuery) -> Result<ListPedidosOutput, ApplicationError> {
        self.list_pedidos.handle(query).await
    }
}

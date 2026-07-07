//! Queries do agregado Produto.

use async_trait::async_trait;

use crate::application::produto::get_produto::GetProduto;
use crate::application::produto::list_produtos::ListProdutos;
use crate::application::shared::cqrs::{Query, QueryHandler};
use crate::application::shared::dto::{ListProdutosOutput, ProdutoOutput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::ProdutoId;

/// Pergunta pelos dados de um produto especifico.
pub struct GetProdutoQuery {
    pub produto_id: ProdutoId,
}

impl Query for GetProdutoQuery {
    type Output = ProdutoOutput;
}

#[async_trait]
impl QueryHandler<GetProdutoQuery> for GetProduto {
    async fn handle(&self, query: GetProdutoQuery) -> Result<ProdutoOutput, ApplicationError> {
        self.execute(query.produto_id).await
    }
}

/// Pergunta pela lista completa de produtos cadastrados.
pub struct ListProdutosQuery;

impl Query for ListProdutosQuery {
    type Output = ListProdutosOutput;
}

#[async_trait]
impl QueryHandler<ListProdutosQuery> for ListProdutos {
    async fn handle(&self, _query: ListProdutosQuery) -> Result<ListProdutosOutput, ApplicationError> {
        self.execute().await
    }
}

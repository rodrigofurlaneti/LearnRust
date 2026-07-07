use std::sync::Arc;

use crate::application::shared::dto::ListProdutosOutput;
use crate::application::shared::errors::ApplicationError;
use crate::application::produto::register_produto::RegisterProduto;
use crate::domain::ProdutoRepository;

/// Caso de uso de listagem de produtos (`GET /produtos`).
pub struct ListProdutos {
    repository: Arc<dyn ProdutoRepository>,
}

impl ListProdutos {
    pub fn new(repository: Arc<dyn ProdutoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<ListProdutosOutput, ApplicationError> {
        let produtos = self.repository.list_all().await?;
        let output = produtos.iter().map(RegisterProduto::to_output).collect();

        Ok(ListProdutosOutput { produtos: output })
    }
}

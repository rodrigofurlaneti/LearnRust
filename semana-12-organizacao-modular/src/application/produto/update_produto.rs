use std::sync::Arc;

use crate::application::shared::dto::{ProdutoOutput, UpdateProdutoInput};
use crate::application::shared::errors::ApplicationError;
use crate::application::produto::register_produto::RegisterProduto;
use crate::domain::{Dinheiro, DomainError, Nome, Produto, ProdutoId, ProdutoRepository};

/// Caso de uso de atualizacao cadastral de produto (`PUT /produtos/:id`).
pub struct UpdateProduto {
    repository: Arc<dyn ProdutoRepository>,
}

impl UpdateProduto {
    pub fn new(repository: Arc<dyn ProdutoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        produto_id: ProdutoId,
        input: UpdateProdutoInput,
    ) -> Result<ProdutoOutput, ApplicationError> {
        let nome = Nome::parse(&input.nome)?;
        let preco = Dinheiro::parse_reais(&input.preco)?;

        let existing_produto = self.find_existing_produto(produto_id).await?;
        let updated_produto = existing_produto.with_updated_data(nome, preco);
        self.repository.update(&updated_produto).await?;

        Ok(RegisterProduto::to_output(&updated_produto))
    }

    async fn find_existing_produto(&self, produto_id: ProdutoId) -> Result<Produto, ApplicationError> {
        self.repository
            .find_by_id(produto_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::ProdutoNotFound))
    }
}

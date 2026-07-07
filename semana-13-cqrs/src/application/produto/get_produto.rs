use std::sync::Arc;

use crate::application::shared::dto::ProdutoOutput;
use crate::application::shared::errors::ApplicationError;
use crate::application::produto::register_produto::RegisterProduto;
use crate::domain::{DomainError, ProdutoId, ProdutoRepository};

/// Caso de uso de consulta de um produto pelo id (`GET /produtos/:id`).
pub struct GetProduto {
    repository: Arc<dyn ProdutoRepository>,
}

impl GetProduto {
    pub fn new(repository: Arc<dyn ProdutoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, produto_id: ProdutoId) -> Result<ProdutoOutput, ApplicationError> {
        let produto = self
            .repository
            .find_by_id(produto_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::ProdutoNotFound))?;

        Ok(RegisterProduto::to_output(&produto))
    }
}

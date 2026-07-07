use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::domain::{DomainError, ProdutoId, ProdutoRepository};

/// Caso de uso de remocao de produto (`DELETE /produtos/:id`). Seguro por
/// design: remover um produto do cadastro nunca invalida pedidos ja
/// feitos, porque `ItemPedido` guarda uma copia do nome/preco (ver
/// `domain::item_pedido`).
pub struct DeleteProduto {
    repository: Arc<dyn ProdutoRepository>,
}

impl DeleteProduto {
    pub fn new(repository: Arc<dyn ProdutoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, produto_id: ProdutoId) -> Result<(), ApplicationError> {
        self.ensure_produto_exists(produto_id).await?;
        self.repository.delete(produto_id).await?;
        Ok(())
    }

    async fn ensure_produto_exists(&self, produto_id: ProdutoId) -> Result<(), ApplicationError> {
        let existing = self.repository.find_by_id(produto_id).await?;
        existing
            .map(|_| ())
            .ok_or(ApplicationError::Domain(DomainError::ProdutoNotFound))
    }
}

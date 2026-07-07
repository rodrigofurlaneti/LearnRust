use async_trait::async_trait;

use crate::domain::shared::errors::DomainError;
use crate::domain::produto::entidade::Produto;
use crate::domain::produto::produto_id::ProdutoId;

/// Porta de saida (Dependency Inversion Principle) do agregado `Produto`,
/// espelhando `ClienteRepository`.
#[async_trait]
pub trait ProdutoRepository: Send + Sync {
    async fn save(&self, produto: &Produto) -> Result<(), DomainError>;
    async fn update(&self, produto: &Produto) -> Result<(), DomainError>;
    async fn delete(&self, id: ProdutoId) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: ProdutoId) -> Result<Option<Produto>, DomainError>;
    async fn list_all(&self) -> Result<Vec<Produto>, DomainError>;
}

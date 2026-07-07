use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DomainError, Produto, ProdutoId, ProdutoRepository};

/// Adapter concreto do port `ProdutoRepository`: mesmo padrao de cache em
/// memoria de `CacheClienteRepository` (HashMap + RwLock).
pub struct CacheProdutoRepository {
    store: Arc<RwLock<HashMap<Uuid, Produto>>>,
}

impl CacheProdutoRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for CacheProdutoRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProdutoRepository for CacheProdutoRepository {
    async fn save(&self, produto: &Produto) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if store.contains_key(&produto.id().as_uuid()) {
            return Err(DomainError::ProdutoAlreadyExists);
        }
        store.insert(produto.id().as_uuid(), produto.clone());
        Ok(())
    }

    async fn update(&self, produto: &Produto) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if !store.contains_key(&produto.id().as_uuid()) {
            return Err(DomainError::ProdutoNotFound);
        }
        store.insert(produto.id().as_uuid(), produto.clone());
        Ok(())
    }

    async fn delete(&self, id: ProdutoId) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        store
            .remove(&id.as_uuid())
            .map(|_| ())
            .ok_or(DomainError::ProdutoNotFound)
    }

    async fn find_by_id(&self, id: ProdutoId) -> Result<Option<Produto>, DomainError> {
        let store = self.store.read().await;
        Ok(store.get(&id.as_uuid()).cloned())
    }

    async fn list_all(&self) -> Result<Vec<Produto>, DomainError> {
        let store = self.store.read().await;
        Ok(store.values().cloned().collect())
    }
}

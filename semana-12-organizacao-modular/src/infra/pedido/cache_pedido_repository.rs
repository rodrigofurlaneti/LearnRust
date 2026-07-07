use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DomainError, Pedido, PedidoId, PedidoRepository};

/// Adapter concreto do port `PedidoRepository`: mesmo padrao de cache em
/// memoria dos demais agregados (HashMap + RwLock). Sem `update` (ver
/// `domain::pedido_repository` para o motivo).
pub struct CachePedidoRepository {
    store: Arc<RwLock<HashMap<Uuid, Pedido>>>,
}

impl CachePedidoRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for CachePedidoRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PedidoRepository for CachePedidoRepository {
    async fn save(&self, pedido: &Pedido) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if store.contains_key(&pedido.id().as_uuid()) {
            return Err(DomainError::PedidoAlreadyExists);
        }
        store.insert(pedido.id().as_uuid(), pedido.clone());
        Ok(())
    }

    async fn delete(&self, id: PedidoId) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        store
            .remove(&id.as_uuid())
            .map(|_| ())
            .ok_or(DomainError::PedidoNotFound)
    }

    async fn find_by_id(&self, id: PedidoId) -> Result<Option<Pedido>, DomainError> {
        let store = self.store.read().await;
        Ok(store.get(&id.as_uuid()).cloned())
    }

    async fn list_all(&self) -> Result<Vec<Pedido>, DomainError> {
        let store = self.store.read().await;
        Ok(store.values().cloned().collect())
    }
}

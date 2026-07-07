use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{Cliente, ClienteId, ClienteRepository, Cpf, DomainError};

/// Adapter concreto do port `ClienteRepository`: mesmo padrao de cache em
/// memoria de `CacheUserRepository` (HashMap + RwLock), simetrico e sem
/// nenhuma dependencia entre os dois "caches" - cada agregado tem o seu.
pub struct CacheClienteRepository {
    store: Arc<RwLock<HashMap<Uuid, Cliente>>>,
}

impl CacheClienteRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn find_by_predicate(&self, matches: impl Fn(&Cliente) -> bool) -> Option<Cliente> {
        let store = self.store.read().await;
        store.values().find(|cliente| matches(cliente)).cloned()
    }
}

impl Default for CacheClienteRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClienteRepository for CacheClienteRepository {
    async fn save(&self, cliente: &Cliente) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if store.contains_key(&cliente.id().as_uuid()) {
            return Err(DomainError::ClienteAlreadyExists);
        }
        store.insert(cliente.id().as_uuid(), cliente.clone());
        Ok(())
    }

    async fn update(&self, cliente: &Cliente) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if !store.contains_key(&cliente.id().as_uuid()) {
            return Err(DomainError::ClienteNotFound);
        }
        store.insert(cliente.id().as_uuid(), cliente.clone());
        Ok(())
    }

    async fn delete(&self, id: ClienteId) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        store
            .remove(&id.as_uuid())
            .map(|_| ())
            .ok_or(DomainError::ClienteNotFound)
    }

    async fn find_by_id(&self, id: ClienteId) -> Result<Option<Cliente>, DomainError> {
        let store = self.store.read().await;
        Ok(store.get(&id.as_uuid()).cloned())
    }

    async fn find_by_documento(&self, documento: &Cpf) -> Result<Option<Cliente>, DomainError> {
        let found = self
            .find_by_predicate(|cliente| cliente.documento() == documento)
            .await;
        Ok(found)
    }

    async fn list_all(&self) -> Result<Vec<Cliente>, DomainError> {
        let store = self.store.read().await;
        Ok(store.values().cloned().collect())
    }
}

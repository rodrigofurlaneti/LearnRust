use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DomainError, Email, User, UserId, UserRepository};

/// Adapter concreto do port `UserRepository` para a Semana 10: em vez de
/// Postgres/SQLite (Semanas 8/9), a "tabela" de usuarios agora e um cache
/// em memoria de processo - um `HashMap` protegido por `RwLock` (leituras
/// concorrentes, escritas exclusivas), o mesmo padrao de estado
/// compartilhado ensinado na Semana 3 (`Arc<Mutex<T>>`), so trocando
/// `Mutex` por `RwLock` porque este caso tem muito mais leitura do que
/// escrita.
///
/// De proposito esta classe NAO sabe nada sobre auditoria - ela so cuida
/// do cache. Quem acrescenta a gravacao no arquivo de historico e o
/// decorator `AuditedUserRepository` (Open/Closed Principle: adicionamos
/// comportamento novo sem tocar nesta classe).
pub struct CacheUserRepository {
    store: Arc<RwLock<HashMap<Uuid, User>>>,
}

impl CacheUserRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn find_by_predicate(&self, matches: impl Fn(&User) -> bool) -> Option<User> {
        let store = self.store.read().await;
        store.values().find(|user| matches(user)).cloned()
    }
}

impl Default for CacheUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for CacheUserRepository {
    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if store.contains_key(&user.id().as_uuid()) {
            return Err(DomainError::UserAlreadyExists);
        }
        store.insert(user.id().as_uuid(), user.clone());
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if !store.contains_key(&user.id().as_uuid()) {
            return Err(DomainError::UserNotFound);
        }
        store.insert(user.id().as_uuid(), user.clone());
        Ok(())
    }

    async fn delete(&self, id: UserId) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        store
            .remove(&id.as_uuid())
            .map(|_| ())
            .ok_or(DomainError::UserNotFound)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let found = self
            .find_by_predicate(|user| user.email() == email)
            .await;
        Ok(found)
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        let store = self.store.read().await;
        Ok(store.get(&id.as_uuid()).cloned())
    }
}

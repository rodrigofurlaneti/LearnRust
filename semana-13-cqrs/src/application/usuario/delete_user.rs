use std::sync::Arc;

use crate::application::shared::errors::ApplicationError;
use crate::domain::{DomainError, UserId, UserRepository};

/// Caso de uso novo na Semana 10: remocao administrativa de usuarios
/// (`DELETE /admin/usuarios/:id`, restrito a `AdminUser`). Confere que o
/// usuario existe antes de tentar remover - devolver `UserNotFound` e mais
/// honesto com quem chama do que um "204" silencioso para um id
/// inexistente.
pub struct DeleteUser {
    repository: Arc<dyn UserRepository>,
}

impl DeleteUser {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: UserId) -> Result<(), ApplicationError> {
        self.ensure_user_exists(user_id).await?;
        self.repository.delete(user_id).await?;
        Ok(())
    }

    async fn ensure_user_exists(&self, user_id: UserId) -> Result<(), ApplicationError> {
        let existing = self.repository.find_by_id(user_id).await?;
        existing
            .map(|_| ())
            .ok_or(ApplicationError::Domain(DomainError::UserNotFound))
    }
}

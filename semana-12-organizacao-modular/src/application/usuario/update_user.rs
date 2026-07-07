use std::sync::Arc;

use crate::application::shared::dto::{UpdateUserInput, UpdateUserOutput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::{DomainError, Email, User, UserId, UserRepository};

/// Caso de uso novo na Semana 10: gestao administrativa de usuarios
/// (`PUT /admin/usuarios/:id`, restrito a `AdminUser` na presentation).
/// So permite trocar o email - trocar senha exigiria reautenticacao/posse
/// da senha atual, e trocar papel e uma decisao de negocio distinta o
/// suficiente para merecer seu proprio caso de uso no futuro (ver README).
pub struct UpdateUser {
    repository: Arc<dyn UserRepository>,
}

impl UpdateUser {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: UserId,
        input: UpdateUserInput,
    ) -> Result<UpdateUserOutput, ApplicationError> {
        let new_email = Email::parse(&input.email)?;
        let existing_user = self.find_existing_user(user_id).await?;
        self.ensure_email_is_available_for(&new_email, user_id).await?;

        let updated_user = existing_user.with_email(new_email);
        self.repository.update(&updated_user).await?;

        Ok(Self::to_output(&updated_user))
    }

    async fn find_existing_user(&self, user_id: UserId) -> Result<User, ApplicationError> {
        self.repository
            .find_by_id(user_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::UserNotFound))
    }

    async fn ensure_email_is_available_for(
        &self,
        email: &Email,
        user_id: UserId,
    ) -> Result<(), ApplicationError> {
        let existing = self.repository.find_by_email(email).await?;
        let belongs_to_someone_else = existing.map(|user| user.id() != user_id).unwrap_or(false);
        if belongs_to_someone_else {
            return Err(ApplicationError::Domain(DomainError::UserAlreadyExists));
        }
        Ok(())
    }

    fn to_output(user: &User) -> UpdateUserOutput {
        UpdateUserOutput {
            user_id: user.id().as_uuid().to_string(),
            email: user.email().as_str().to_string(),
            role: user.role().as_str().to_string(),
        }
    }
}

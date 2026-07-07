use std::sync::Arc;

use crate::application::shared::dto::CurrentUserOutput;
use crate::application::shared::errors::ApplicationError;
use crate::domain::{DomainError, UserId, UserRepository};

/// Caso de uso por tras de `GET /me`. Recebe o `UserId` ja validado pelo
/// extractor de autenticacao (o token foi conferido antes disso, na
/// presentation) e busca os dados atuais do usuario - nunca confia so no
/// que estava no token, sempre revalida contra o repositorio. Herdado sem
/// alteracoes da Semana 9.
pub struct GetCurrentUser {
    repository: Arc<dyn UserRepository>,
}

impl GetCurrentUser {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: UserId) -> Result<CurrentUserOutput, ApplicationError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::InvalidCredentials))?;

        Ok(CurrentUserOutput {
            user_id: user.id().as_uuid().to_string(),
            email: user.email().as_str().to_string(),
            role: user.role().as_str().to_string(),
        })
    }
}

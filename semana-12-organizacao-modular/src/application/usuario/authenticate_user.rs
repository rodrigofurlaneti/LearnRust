use std::sync::Arc;

use crate::application::shared::dto::{AuthenticateUserInput, AuthenticateUserOutput};
use crate::application::shared::errors::ApplicationError;
use crate::application::shared::token_service::TokenService;
use crate::domain::{DomainError, Email, PasswordHasher, PlainPassword, User, UserRepository};

/// Caso de uso de login. Observacao de seguranca (regra de negocio
/// implicita): tanto "email nao existe" quanto "senha errada" resultam no
/// mesmo `DomainError::InvalidCredentials`, para nao vazar quais emails
/// estao cadastrados. Herdado sem alteracoes da Semana 9.
pub struct AuthenticateUser {
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
}

impl AuthenticateUser {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            repository,
            hasher,
            tokens,
        }
    }

    pub async fn execute(
        &self,
        input: AuthenticateUserInput,
    ) -> Result<AuthenticateUserOutput, ApplicationError> {
        let email = Email::parse(&input.email)?;
        let plain_password = PlainPassword::parse(&input.password)?;

        let user = self.find_registered_user(&email).await?;
        self.ensure_password_matches(&user, &plain_password).await?;
        let token = self.issue_token_for(&user)?;

        Ok(AuthenticateUserOutput {
            access_token: token.0,
        })
    }

    async fn find_registered_user(&self, email: &Email) -> Result<User, ApplicationError> {
        self.repository
            .find_by_email(email)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::InvalidCredentials))
    }

    async fn ensure_password_matches(
        &self,
        user: &User,
        candidate: &PlainPassword,
    ) -> Result<(), ApplicationError> {
        let matches = user.matches_password(candidate, self.hasher.as_ref()).await;
        if !matches {
            return Err(ApplicationError::Domain(DomainError::InvalidCredentials));
        }
        Ok(())
    }

    fn issue_token_for(
        &self,
        user: &User,
    ) -> Result<crate::application::shared::token_service::AccessToken, ApplicationError> {
        self.tokens
            .issue(user.id(), user.role())
            .map_err(ApplicationError::Unexpected)
    }
}

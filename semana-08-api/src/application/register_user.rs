use std::sync::Arc;

use crate::application::dto::{RegisterUserInput, RegisterUserOutput};
use crate::application::errors::ApplicationError;
use crate::domain::{DomainError, Email, PasswordHasher, PlainPassword, User, UserRepository};

/// Caso de uso (SRP: uma única razão para mudar — a política de cadastro).
/// Depende só de abstrações do domínio (DIP), nunca de Postgres/bcrypt
/// diretamente.
pub struct RegisterUser {
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl RegisterUser {
    pub fn new(repository: Arc<dyn UserRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { repository, hasher }
    }

    pub async fn execute(
        &self,
        input: RegisterUserInput,
    ) -> Result<RegisterUserOutput, ApplicationError> {
        let email = Email::parse(&input.email)?;
        let plain_password = PlainPassword::parse(&input.password)?;

        self.ensure_email_is_available(&email).await?;
        let user = self.create_user(email, plain_password).await;
        self.repository.save(&user).await?;

        Ok(Self::to_output(&user))
    }

    async fn ensure_email_is_available(&self, email: &Email) -> Result<(), ApplicationError> {
        let existing = self.repository.find_by_email(email).await?;
        if existing.is_some() {
            return Err(ApplicationError::Domain(DomainError::UserAlreadyExists));
        }
        Ok(())
    }

    async fn create_user(&self, email: Email, plain_password: PlainPassword) -> User {
        let hashed_password = self.hasher.hash(&plain_password).await;
        User::register(email, hashed_password)
    }

    fn to_output(user: &User) -> RegisterUserOutput {
        RegisterUserOutput {
            user_id: user.id().as_uuid().to_string(),
            email: user.email().as_str().to_string(),
        }
    }
}

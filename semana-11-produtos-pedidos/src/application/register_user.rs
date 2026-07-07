use std::sync::Arc;

use crate::application::dto::{RegisterUserInput, RegisterUserOutput};
use crate::application::errors::ApplicationError;
use crate::domain::{DomainError, Email, PasswordHasher, PlainPassword, User, UserRepository};

/// Caso de uso (SRP: uma unica razao para mudar - a politica de cadastro).
/// Depende so de abstracoes do dominio (DIP), nunca da implementacao
/// concreta de cache/auditoria - isso e resolvido no composition root
/// (`main.rs`), que injeta um `UserRepository` ja decorado com auditoria.
/// Este caso de uso, portanto, nao sabe (e nao precisa saber) que a
/// gravacao dele vai parar num arquivo de historico - responsabilidade
/// unica preservada.
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
            role: user.role().as_str().to_string(),
        }
    }
}

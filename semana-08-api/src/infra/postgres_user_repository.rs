use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::{DomainError, Email, HashedPassword, User, UserId, UserRepository};

/// Adapter concreto do port `UserRepository`, usando SQLx + Postgres.
/// É a única classe do sistema que sabe o nome das colunas da tabela `users`.
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_user(row: sqlx::postgres::PgRow) -> User {
        let id: uuid::Uuid = row.get("id");
        let email: String = row.get("email");
        let password_hash: String = row.get("password_hash");

        User::reconstitute(
            UserId::from_uuid(id),
            Email::parse(&email).expect("email persistido deveria já ser válido"),
            HashedPassword::from_hash(password_hash),
        )
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn save(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
            .bind(user.id().as_uuid())
            .bind(user.email().as_str())
            .bind(user.password_hash().as_str())
            .execute(&self.pool)
            .await
            .map_err(|_| DomainError::UserAlreadyExists)?;

        Ok(())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let row = sqlx::query("SELECT id, email, password_hash FROM users WHERE email = $1")
            .bind(email.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| DomainError::InvalidCredentials)?;

        Ok(row.map(Self::row_to_user))
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        let row = sqlx::query("SELECT id, email, password_hash FROM users WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| DomainError::InvalidCredentials)?;

        Ok(row.map(Self::row_to_user))
    }
}

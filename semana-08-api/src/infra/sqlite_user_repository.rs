use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::{DomainError, Email, HashedPassword, User, UserId, UserRepository};

/// Adapter para desenvolvimento e testes: mesmo contrato `UserRepository`
/// do Postgres, porem apoiado em SQLite totalmente em memoria. O schema e
/// criado do zero toda vez que o pool e montado (ou seja, toda vez que a
/// aplicacao sobe) - nao existe arquivo em disco, nada persiste entre
/// execucoes. Ideal para "cargo run" sem depender de Docker/Postgres, e
/// para testes de integracao rapidos e isolados uns dos outros.
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn row_to_user(row: sqlx::sqlite::SqliteRow) -> User {
        let id: String = row.get("id");
        let email: String = row.get("email");
        let password_hash: String = row.get("password_hash");

        User::reconstitute(
            UserId::from_uuid(Uuid::parse_str(&id).expect("uuid invalido salvo no banco")),
            Email::parse(&email).expect("email ja validado antes de ser salvo"),
            HashedPassword::from_hash(password_hash),
        )
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn save(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)")
            .bind(user.id().as_uuid().to_string())
            .bind(user.email().as_str())
            .bind(user.password_hash().as_str())
            .execute(&self.pool)
            .await
            .map_err(|_| DomainError::UserAlreadyExists)?;

        Ok(())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let row = sqlx::query("SELECT id, email, password_hash FROM users WHERE email = ?")
            .bind(email.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| DomainError::InvalidCredentials)?;

        Ok(row.map(Self::row_to_user))
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        let row = sqlx::query("SELECT id, email, password_hash FROM users WHERE id = ?")
            .bind(id.as_uuid().to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| DomainError::InvalidCredentials)?;

        Ok(row.map(Self::row_to_user))
    }
}

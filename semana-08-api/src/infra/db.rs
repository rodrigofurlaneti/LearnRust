use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{PgPool, SqlitePool};

/// Detalhe de infraestrutura: como o pool de conexoes com Postgres e
/// criado. Nada nas camadas de dominio/aplicacao sabe que isto existe.
pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("falha ao conectar no PostgreSQL")
}

/// Schema do banco "de mentira": criado em memoria, do zero, toda vez que
/// a aplicacao (ou um teste) sobe. Nao ha migration separada porque nao
/// ha nada para migrar - o banco nasce e morre com o processo.
const SQLITE_USERS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

/// Cria um banco SQLite 100% em memoria e ja aplica o schema.
///
/// Nota importante: usamos `max_connections(1)` de proposito. O SQLite em
/// modo `:memory:` cria um banco novo e isolado a cada conexao: se o pool
/// abrisse varias conexoes, cada uma enxergaria dados diferentes. Com uma
/// unica conexao viva durante toda a vida do pool, todas as operacoes da
/// aplicacao (ou de um teste) conversam com o mesmo banco.
pub async fn create_in_memory_sqlite_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("falha ao criar banco sqlite em memoria");

    sqlx::query(SQLITE_USERS_SCHEMA)
        .execute(&pool)
        .await
        .expect("falha ao criar schema do banco em memoria");

    pool
}

use std::sync::Arc;

use auth_api::application::authenticate_user::AuthenticateUser;
use auth_api::application::register_user::RegisterUser;
use auth_api::domain::UserRepository;
use auth_api::infra::bcrypt_hasher::BcryptPasswordHasher;
use auth_api::infra::db::{create_in_memory_sqlite_pool, create_pool};
use auth_api::infra::jwt_token_service::JwtTokenService;
use auth_api::infra::postgres_user_repository::PostgresUserRepository;
use auth_api::infra::sqlite_user_repository::SqliteUserRepository;
use auth_api::presentation::routes::build_router;
use auth_api::presentation::state::AppState;

/// Composition root: o UNICO lugar do sistema onde dominio, aplicacao,
/// infra e presentation se encontram. A escolha de qual adapter de
/// `UserRepository` usar (Postgres real ou SQLite em memoria) tambem
/// acontece so aqui - application e domain nem sabem que essa escolha
/// existe.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string());

    let repository = build_repository().await;
    let hasher = Arc::new(BcryptPasswordHasher::new(12));
    let tokens = Arc::new(JwtTokenService::new(jwt_secret, 3600));

    let register_user = Arc::new(RegisterUser::new(repository.clone(), hasher.clone()));
    let authenticate_user = Arc::new(AuthenticateUser::new(repository, hasher, tokens));

    let state = AppState {
        register_user,
        authenticate_user,
    };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("nao foi possivel abrir a porta 3000");

    tracing::info!("auth-api ouvindo em http://0.0.0.0:3000");
    axum::serve(listener, app)
        .await
        .expect("erro ao servir a aplicacao");
}

/// Le `DATABASE_BACKEND` do ambiente para decidir qual adapter de
/// persistencia usar. Sem essa variavel, o padrao e "memory" - assim
/// `cargo run` funciona de cara, sem precisar subir Postgres via Docker.
/// Em producao, defina `DATABASE_BACKEND=postgres` e `DATABASE_URL`.
async fn build_repository() -> Arc<dyn UserRepository> {
    let backend = std::env::var("DATABASE_BACKEND").unwrap_or_else(|_| "memory".to_string());

    match backend.as_str() {
        "postgres" => {
            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL e obrigatorio quando DATABASE_BACKEND=postgres");
            let pool = create_pool(&database_url).await;
            Arc::new(PostgresUserRepository::new(pool))
        }
        "memory" => {
            tracing::warn!(
                "DATABASE_BACKEND=memory: usando SQLite em memoria, os dados somem ao reiniciar"
            );
            let pool = create_in_memory_sqlite_pool().await;
            Arc::new(SqliteUserRepository::new(pool))
        }
        other => panic!("DATABASE_BACKEND invalido: \"{other}\" (use \"memory\" ou \"postgres\")"),
    }
}

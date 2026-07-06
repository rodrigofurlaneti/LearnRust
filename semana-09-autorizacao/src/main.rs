use std::net::SocketAddr;
use std::sync::Arc;

use auth_api::application::authenticate_user::AuthenticateUser;
use auth_api::application::get_current_user::GetCurrentUser;
use auth_api::application::register_user::RegisterUser;
use auth_api::application::token_service::TokenService;
use auth_api::domain::{Email, PasswordHasher, PlainPassword, Role, User, UserRepository};
use auth_api::infra::bcrypt_hasher::BcryptPasswordHasher;
use auth_api::infra::db::{create_in_memory_sqlite_pool, create_pool};
use auth_api::infra::jwt_token_service::JwtTokenService;
use auth_api::infra::postgres_user_repository::PostgresUserRepository;
use auth_api::infra::sqlite_user_repository::SqliteUserRepository;
use auth_api::presentation::routes::build_router;
use auth_api::presentation::state::AppState;

const SEED_ADMIN_EMAIL: &str = "admin@example.com";
const SEED_ADMIN_PASSWORD: &str = "AdminForte123";

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
    let backend = std::env::var("DATABASE_BACKEND").unwrap_or_else(|_| "memory".to_string());

    let repository = build_repository(&backend).await;
    let hasher: Arc<dyn PasswordHasher> = Arc::new(BcryptPasswordHasher::new(12));
    let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(jwt_secret, 3600));

    if backend == "memory" {
        seed_default_admin(&repository, &hasher).await;
    }

    let register_user = Arc::new(RegisterUser::new(repository.clone(), hasher.clone()));
    let authenticate_user = Arc::new(AuthenticateUser::new(
        repository.clone(),
        hasher,
        tokens.clone(),
    ));
    let get_current_user = Arc::new(GetCurrentUser::new(repository));

    let state = AppState {
        register_user,
        authenticate_user,
        get_current_user,
        tokens,
    };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("nao foi possivel abrir a porta 3000");

    tracing::info!("auth-api ouvindo em http://0.0.0.0:3000");
    // `with_connect_info` (em vez de so `into_make_service()`) e o que da
    // ao tower_governor o IP de origem de cada conexao, usado como chave
    // do rate limiter em `routes.rs`.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("erro ao servir a aplicacao");
}

/// Le `DATABASE_BACKEND` do ambiente para decidir qual adapter de
/// persistencia usar. Sem essa variavel, o padrao e "memory" - assim
/// `cargo run` funciona de cara, sem precisar subir Postgres via Docker.
/// Em producao, defina `DATABASE_BACKEND=postgres` e `DATABASE_URL`.
async fn build_repository(backend: &str) -> Arc<dyn UserRepository> {
    match backend {
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

/// So roda no backend em memoria (nunca em Postgres/producao): cria um
/// usuario Role::Admin conhecido, para dar para testar a rota
/// `GET /admin/ping` sem precisar de nenhuma ferramenta externa de
/// provisionamento. Usa `User::register_with_role` (nao `User::register`)
/// de proposito - o endpoint publico de cadastro jamais cria admins.
async fn seed_default_admin(repository: &Arc<dyn UserRepository>, hasher: &Arc<dyn PasswordHasher>) {
    let email = Email::parse(SEED_ADMIN_EMAIL).expect("email de seed invalido");

    let already_exists = repository
        .find_by_email(&email)
        .await
        .ok()
        .flatten()
        .is_some();
    if already_exists {
        return;
    }

    let password =
        PlainPassword::parse(SEED_ADMIN_PASSWORD).expect("senha de seed nao atende a politica");
    let hashed = hasher.hash(&password).await;
    let admin = User::register_with_role(email, hashed, Role::Admin);

    if repository.save(&admin).await.is_ok() {
        tracing::warn!(
            "usuario admin de exemplo criado: {} / {} (so no backend memory)",
            SEED_ADMIN_EMAIL,
            SEED_ADMIN_PASSWORD
        );
    }
}

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use cadastro_api::application::audit_logger::AuditLogger;
use cadastro_api::application::authenticate_user::AuthenticateUser;
use cadastro_api::application::delete_cliente::DeleteCliente;
use cadastro_api::application::delete_user::DeleteUser;
use cadastro_api::application::get_cliente::GetCliente;
use cadastro_api::application::get_current_user::GetCurrentUser;
use cadastro_api::application::list_clientes::ListClientes;
use cadastro_api::application::register_cliente::RegisterCliente;
use cadastro_api::application::register_user::RegisterUser;
use cadastro_api::application::token_service::TokenService;
use cadastro_api::application::update_cliente::UpdateCliente;
use cadastro_api::application::update_user::UpdateUser;
use cadastro_api::domain::{ClienteRepository, Email, PasswordHasher, PlainPassword, Role, User, UserRepository};
use cadastro_api::infra::audited_cliente_repository::AuditedClienteRepository;
use cadastro_api::infra::audited_user_repository::AuditedUserRepository;
use cadastro_api::infra::bcrypt_hasher::BcryptPasswordHasher;
use cadastro_api::infra::cache_cliente_repository::CacheClienteRepository;
use cadastro_api::infra::cache_user_repository::CacheUserRepository;
use cadastro_api::infra::file_audit_logger::FileAuditLogger;
use cadastro_api::infra::jwt_token_service::JwtTokenService;
use cadastro_api::presentation::routes::build_router;
use cadastro_api::presentation::state::AppState;

const SEED_ADMIN_EMAIL: &str = "admin@example.com";
const SEED_ADMIN_PASSWORD: &str = "AdminForte123";

/// Composition root: o UNICO lugar do projeto onde dominio, aplicacao,
/// infra e presentation se encontram - e tambem o unico lugar que sabe
/// que "usuario"/"cliente" moram num cache em memoria e que toda escrita
/// vira uma linha no arquivo de auditoria. Nada fora daqui conhece
/// `CacheUserRepository`, `AuditedUserRepository` ou `FileAuditLogger`.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string());
    let audit_log_path =
        std::env::var("AUDIT_LOG_PATH").unwrap_or_else(|_| "auditoria.txt".to_string());

    let audit: Arc<dyn AuditLogger> = Arc::new(FileAuditLogger::start(PathBuf::from(&audit_log_path)));
    let hasher: Arc<dyn PasswordHasher> = Arc::new(BcryptPasswordHasher::new(12));
    let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(jwt_secret, 3600));

    let user_repository = build_audited_user_repository(audit.clone());
    let cliente_repository = build_audited_cliente_repository(audit);

    seed_default_admin(&user_repository, &hasher).await;

    let state = build_app_state(user_repository, cliente_repository, hasher, tokens);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("nao foi possivel abrir a porta 3000");

    tracing::info!("cadastro-api ouvindo em http://0.0.0.0:3000");
    tracing::info!("historico de auditoria sendo gravado em {}", audit_log_path);
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

/// Monta a "tabela" de usuarios da semana: cache em memoria
/// (`CacheUserRepository`) decorado com auditoria em arquivo
/// (`AuditedUserRepository`). So esta funcao sabe que a decoracao existe -
/// o resto do sistema recebe apenas um `Arc<dyn UserRepository>`.
fn build_audited_user_repository(audit: Arc<dyn AuditLogger>) -> Arc<dyn UserRepository> {
    let cache = Arc::new(CacheUserRepository::new());
    Arc::new(AuditedUserRepository::new(cache, audit))
}

fn build_audited_cliente_repository(audit: Arc<dyn AuditLogger>) -> Arc<dyn ClienteRepository> {
    let cache = Arc::new(CacheClienteRepository::new());
    Arc::new(AuditedClienteRepository::new(cache, audit))
}

fn build_app_state(
    user_repository: Arc<dyn UserRepository>,
    cliente_repository: Arc<dyn ClienteRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
) -> AppState {
    let register_user = Arc::new(RegisterUser::new(user_repository.clone(), hasher.clone()));
    let authenticate_user = Arc::new(AuthenticateUser::new(
        user_repository.clone(),
        hasher.clone(),
        tokens.clone(),
    ));
    let get_current_user = Arc::new(GetCurrentUser::new(user_repository.clone()));
    let update_user = Arc::new(UpdateUser::new(user_repository.clone()));
    let delete_user = Arc::new(DeleteUser::new(user_repository));

    let register_cliente = Arc::new(RegisterCliente::new(cliente_repository.clone()));
    let get_cliente = Arc::new(GetCliente::new(cliente_repository.clone()));
    let list_clientes = Arc::new(ListClientes::new(cliente_repository.clone()));
    let update_cliente = Arc::new(UpdateCliente::new(cliente_repository.clone()));
    let delete_cliente = Arc::new(DeleteCliente::new(cliente_repository));

    AppState {
        register_user,
        authenticate_user,
        get_current_user,
        update_user,
        delete_user,
        register_cliente,
        get_cliente,
        list_clientes,
        update_cliente,
        delete_cliente,
        tokens,
    }
}

/// Cria um usuario Role::Admin conhecido, para dar para testar as rotas
/// `/admin/*` sem precisar de nenhuma ferramenta externa de
/// provisionamento - mesma ideia da Semana 9, agora persistindo no cache
/// (via o repositorio ja decorado com auditoria: o proprio seed vira a
/// primeira linha do arquivo de historico). Usa `User::register_with_role`
/// (nao `User::register`) de proposito - o endpoint publico de cadastro
/// jamais cria admins.
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
            "usuario admin de exemplo criado: {} / {}",
            SEED_ADMIN_EMAIL,
            SEED_ADMIN_PASSWORD
        );
    }
}

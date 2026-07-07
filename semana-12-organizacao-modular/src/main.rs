use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cadastro_api::application::shared::audit_logger::AuditLogger;
use cadastro_api::application::usuario::authenticate_user::AuthenticateUser;
use cadastro_api::application::cliente::delete_cliente::DeleteCliente;
use cadastro_api::application::pedido::delete_pedido::DeletePedido;
use cadastro_api::application::produto::delete_produto::DeleteProduto;
use cadastro_api::application::usuario::delete_user::DeleteUser;
use cadastro_api::application::cliente::get_cliente::GetCliente;
use cadastro_api::application::usuario::get_current_user::GetCurrentUser;
use cadastro_api::application::pedido::get_pedido::GetPedido;
use cadastro_api::application::produto::get_produto::GetProduto;
use cadastro_api::application::cliente::list_clientes::ListClientes;
use cadastro_api::application::pedido::list_pedidos::ListPedidos;
use cadastro_api::application::produto::list_produtos::ListProdutos;
use cadastro_api::application::cliente::register_cliente::RegisterCliente;
use cadastro_api::application::pedido::register_pedido::RegisterPedido;
use cadastro_api::application::produto::register_produto::RegisterProduto;
use cadastro_api::application::usuario::register_user::RegisterUser;
use cadastro_api::application::shared::token_service::TokenService;
use cadastro_api::application::cliente::update_cliente::UpdateCliente;
use cadastro_api::application::produto::update_produto::UpdateProduto;
use cadastro_api::application::usuario::update_user::UpdateUser;
use cadastro_api::domain::{
    ClienteRepository, Email, PasswordHasher, PedidoRepository, PlainPassword, ProdutoRepository,
    Role, User, UserRepository,
};
use cadastro_api::infra::shared::audit_log_replayer::AuditLogReplayer;
use cadastro_api::infra::cliente::audited_cliente_repository::AuditedClienteRepository;
use cadastro_api::infra::pedido::audited_pedido_repository::AuditedPedidoRepository;
use cadastro_api::infra::produto::audited_produto_repository::AuditedProdutoRepository;
use cadastro_api::infra::usuario::audited_user_repository::AuditedUserRepository;
use cadastro_api::infra::usuario::bcrypt_hasher::BcryptPasswordHasher;
use cadastro_api::infra::cliente::cache_cliente_repository::CacheClienteRepository;
use cadastro_api::infra::pedido::cache_pedido_repository::CachePedidoRepository;
use cadastro_api::infra::produto::cache_produto_repository::CacheProdutoRepository;
use cadastro_api::infra::usuario::cache_user_repository::CacheUserRepository;
use cadastro_api::infra::shared::file_audit_logger::FileAuditLogger;
use cadastro_api::infra::usuario::jwt_token_service::JwtTokenService;
use cadastro_api::presentation::shared::routes::build_router;
use cadastro_api::presentation::shared::state::AppState;

const SEED_ADMIN_EMAIL: &str = "admin@example.com";
const SEED_ADMIN_PASSWORD: &str = "AdminForte123";

/// Composition root: o UNICO lugar do projeto onde dominio, aplicacao,
/// infra e presentation se encontram - e tambem o unico lugar que sabe
/// que "usuario"/"cliente"/"produto"/"pedido" moram num cache em memoria,
/// que toda escrita vira uma linha no arquivo de auditoria, E que esse
/// mesmo arquivo e lido de volta no boot para reidratar o cache (novo na
/// Semana 11). Nada fora daqui conhece `Cache*Repository`,
/// `Audited*Repository`, `FileAuditLogger` ou `AuditLogReplayer`.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string());
    let audit_log_path =
        PathBuf::from(std::env::var("AUDIT_LOG_PATH").unwrap_or_else(|_| "auditoria.txt".to_string()));

    let (user_repository, cliente_repository, produto_repository, pedido_repository, _audit) =
        build_repositories_reidratadas(&audit_log_path).await;

    let hasher: Arc<dyn PasswordHasher> = Arc::new(BcryptPasswordHasher::new(12));
    let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(jwt_secret, 3600));

    seed_default_admin(&user_repository, &hasher).await;

    let state = build_app_state(
        user_repository,
        cliente_repository,
        produto_repository,
        pedido_repository,
        hasher,
        tokens,
    );
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("nao foi possivel abrir a porta 3000");

    tracing::info!("cadastro-api ouvindo em http://0.0.0.0:3000");
    tracing::info!("historico/estado de auditoria em {:?}", audit_log_path);
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

type Repositorios = (
    Arc<dyn UserRepository>,
    Arc<dyn ClienteRepository>,
    Arc<dyn ProdutoRepository>,
    Arc<dyn PedidoRepository>,
    Arc<dyn AuditLogger>,
);

/// Monta as quatro "tabelas" da semana como cache em memoria, REIDRATA
/// esse cache a partir do arquivo de auditoria existente (se houver) e so
/// DEPOIS decora cada repositorio com auditoria - nessa ordem exata:
///
/// 1. Cria os caches puros e vazios.
/// 2. `AuditLogReplayer::replay` le o arquivo (se existir) e repovoa os
///    caches, chamando save/update/delete diretamente nos repositorios
///    NAO decorados - reaplicar historico nao deveria gerar historico novo.
/// 3. So agora criamos o `FileAuditLogger` (que abre o arquivo em modo
///    append para os proximos eventos) e envolvemos cada cache com seu
///    decorator de auditoria - a partir daqui, toda escrita nova volta a
///    virar uma linha no arquivo.
async fn build_repositories_reidratadas(audit_log_path: &Path) -> Repositorios {
    let user_cache: Arc<dyn UserRepository> = Arc::new(CacheUserRepository::new());
    let cliente_cache: Arc<dyn ClienteRepository> = Arc::new(CacheClienteRepository::new());
    let produto_cache: Arc<dyn ProdutoRepository> = Arc::new(CacheProdutoRepository::new());
    let pedido_cache: Arc<dyn PedidoRepository> = Arc::new(CachePedidoRepository::new());

    AuditLogReplayer::replay(
        audit_log_path,
        &user_cache,
        &cliente_cache,
        &produto_cache,
        &pedido_cache,
    )
    .await;

    let audit: Arc<dyn AuditLogger> = Arc::new(FileAuditLogger::start(audit_log_path.to_path_buf()));

    let user_repository: Arc<dyn UserRepository> =
        Arc::new(AuditedUserRepository::new(user_cache, audit.clone()));
    let cliente_repository: Arc<dyn ClienteRepository> =
        Arc::new(AuditedClienteRepository::new(cliente_cache, audit.clone()));
    let produto_repository: Arc<dyn ProdutoRepository> =
        Arc::new(AuditedProdutoRepository::new(produto_cache, audit.clone()));
    let pedido_repository: Arc<dyn PedidoRepository> =
        Arc::new(AuditedPedidoRepository::new(pedido_cache, audit.clone()));

    (user_repository, cliente_repository, produto_repository, pedido_repository, audit)
}

#[allow(clippy::too_many_arguments)]
fn build_app_state(
    user_repository: Arc<dyn UserRepository>,
    cliente_repository: Arc<dyn ClienteRepository>,
    produto_repository: Arc<dyn ProdutoRepository>,
    pedido_repository: Arc<dyn PedidoRepository>,
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
    let delete_cliente = Arc::new(DeleteCliente::new(cliente_repository.clone()));

    let register_produto = Arc::new(RegisterProduto::new(produto_repository.clone()));
    let get_produto = Arc::new(GetProduto::new(produto_repository.clone()));
    let list_produtos = Arc::new(ListProdutos::new(produto_repository.clone()));
    let update_produto = Arc::new(UpdateProduto::new(produto_repository.clone()));
    let delete_produto = Arc::new(DeleteProduto::new(produto_repository.clone()));

    let register_pedido = Arc::new(RegisterPedido::new(
        cliente_repository,
        produto_repository,
        pedido_repository.clone(),
    ));
    let get_pedido = Arc::new(GetPedido::new(pedido_repository.clone()));
    let list_pedidos = Arc::new(ListPedidos::new(pedido_repository.clone()));
    let delete_pedido = Arc::new(DeletePedido::new(pedido_repository));

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
        register_produto,
        get_produto,
        list_produtos,
        update_produto,
        delete_produto,
        register_pedido,
        get_pedido,
        list_pedidos,
        delete_pedido,
        tokens,
    }
}

/// Cria um usuario Role::Admin conhecido, para dar para testar as rotas
/// `/admin/*` sem precisar de nenhuma ferramenta externa de
/// provisionamento. Como agora o cache pode ja ter sido reidratado do
/// arquivo de auditoria, a checagem "ja existe?" antes de semear tambem
/// serve para nao duplicar o admin a cada restart - o seed so acontece
/// mesmo na primeira vez que o processo sobe (arquivo de auditoria vazio
/// ou inexistente).
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

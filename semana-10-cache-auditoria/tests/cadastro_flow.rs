use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cadastro_api::application::audit_logger::AuditLogger;
use cadastro_api::application::authenticate_user::AuthenticateUser;
use cadastro_api::application::delete_cliente::DeleteCliente;
use cadastro_api::application::delete_user::DeleteUser;
use cadastro_api::application::dto::{
    AuthenticateUserInput, RegisterClienteInput, RegisterUserInput, UpdateClienteInput,
    UpdateUserInput,
};
use cadastro_api::application::get_cliente::GetCliente;
use cadastro_api::application::get_current_user::GetCurrentUser;
use cadastro_api::application::list_clientes::ListClientes;
use cadastro_api::application::register_cliente::RegisterCliente;
use cadastro_api::application::register_user::RegisterUser;
use cadastro_api::application::update_cliente::UpdateCliente;
use cadastro_api::application::update_user::UpdateUser;
use cadastro_api::domain::{ClienteRepository, UserRepository};
use cadastro_api::infra::audited_cliente_repository::AuditedClienteRepository;
use cadastro_api::infra::audited_user_repository::AuditedUserRepository;
use cadastro_api::infra::bcrypt_hasher::BcryptPasswordHasher;
use cadastro_api::infra::cache_cliente_repository::CacheClienteRepository;
use cadastro_api::infra::cache_user_repository::CacheUserRepository;
use cadastro_api::infra::file_audit_logger::FileAuditLogger;
use cadastro_api::infra::jwt_token_service::JwtTokenService;

/// Cada teste monta seu proprio cache em memoria e seu proprio arquivo de
/// auditoria (isolados dos outros testes, mesmo raciocinio da Semana 9
/// com bancos SQLite em memoria isolados por teste).
struct TestEnv {
    register_user: RegisterUser,
    authenticate_user: AuthenticateUser,
    get_current_user: GetCurrentUser,
    update_user: UpdateUser,
    delete_user: DeleteUser,
    register_cliente: RegisterCliente,
    get_cliente: GetCliente,
    list_clientes: ListClientes,
    update_cliente: UpdateCliente,
    delete_cliente: DeleteCliente,
    audit_log_path: PathBuf,
}

fn unique_audit_path(test_name: &str) -> PathBuf {
    let file_name = format!("auditoria_teste_{}_{}.txt", test_name, uuid::Uuid::new_v4());
    std::env::temp_dir().join(file_name)
}

fn build_test_env(test_name: &str) -> TestEnv {
    let audit_log_path = unique_audit_path(test_name);
    let audit: Arc<dyn AuditLogger> = Arc::new(FileAuditLogger::start(audit_log_path.clone()));

    let user_cache = Arc::new(CacheUserRepository::new());
    let user_repository: Arc<dyn UserRepository> =
        Arc::new(AuditedUserRepository::new(user_cache, audit.clone()));

    let cliente_cache = Arc::new(CacheClienteRepository::new());
    let cliente_repository: Arc<dyn ClienteRepository> =
        Arc::new(AuditedClienteRepository::new(cliente_cache, audit));

    let hasher = Arc::new(BcryptPasswordHasher::new(4)); // custo baixo: testes rapidos
    let tokens = Arc::new(JwtTokenService::new("test-secret".to_string(), 3600));

    TestEnv {
        register_user: RegisterUser::new(user_repository.clone(), hasher.clone()),
        authenticate_user: AuthenticateUser::new(
            user_repository.clone(),
            hasher.clone(),
            tokens.clone(),
        ),
        get_current_user: GetCurrentUser::new(user_repository.clone()),
        update_user: UpdateUser::new(user_repository.clone()),
        delete_user: DeleteUser::new(user_repository),
        register_cliente: RegisterCliente::new(cliente_repository.clone()),
        get_cliente: GetCliente::new(cliente_repository.clone()),
        list_clientes: ListClientes::new(cliente_repository.clone()),
        update_cliente: UpdateCliente::new(cliente_repository.clone()),
        delete_cliente: DeleteCliente::new(cliente_repository),
        audit_log_path,
    }
}

/// A gravacao no arquivo e "fire-and-forget" (ver README, secao 7): o
/// teste da uma folga pequena para a task de background do
/// `FileAuditLogger` esvaziar o canal antes de ler o arquivo.
async fn read_audit_log(path: &PathBuf) -> String {
    tokio::time::sleep(Duration::from_millis(100)).await;
    tokio::fs::read_to_string(path).await.unwrap_or_default()
}

#[tokio::test]
async fn registra_atualiza_e_remove_usuario_gerando_auditoria() {
    let env = build_test_env("usuario_crud");

    let register_output = env
        .register_user
        .execute(RegisterUserInput {
            email: "rodrigo@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let login_output = env
        .authenticate_user
        .execute(AuthenticateUserInput {
            email: "rodrigo@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("login deveria funcionar");
    assert!(!login_output.access_token.is_empty());

    let user_uuid = uuid::Uuid::parse_str(&register_output.user_id).expect("uuid valido");
    let user_id = cadastro_api::domain::UserId::from_uuid(user_uuid);

    let profile = env
        .get_current_user
        .execute(user_id)
        .await
        .expect("busca do usuario deveria funcionar");
    assert_eq!(profile.email, "rodrigo@example.com");

    env.update_user
        .execute(
            user_id,
            UpdateUserInput {
                email: "rodrigo.novo@example.com".to_string(),
            },
        )
        .await
        .expect("atualizacao deveria funcionar");

    env.delete_user
        .execute(user_id)
        .await
        .expect("remocao deveria funcionar");

    let audit_log = read_audit_log(&env.audit_log_path).await;
    assert!(audit_log.contains("USUARIO") && audit_log.contains("INSERT"));
    assert!(audit_log.contains("UPDATE") && audit_log.contains("rodrigo.novo@example.com"));
    assert!(audit_log.contains("DELETE"));
}

#[tokio::test]
async fn update_user_falha_para_usuario_inexistente() {
    let env = build_test_env("usuario_inexistente");
    let id_aleatorio = cadastro_api::domain::UserId::new();

    let result = env
        .update_user
        .execute(
            id_aleatorio,
            UpdateUserInput {
                email: "ninguem@example.com".to_string(),
            },
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn cadastra_consulta_lista_atualiza_e_remove_cliente_gerando_auditoria() {
    let env = build_test_env("cliente_crud");

    let cliente = env
        .register_cliente
        .execute(RegisterClienteInput {
            nome: "Rodrigo Furlaneti".to_string(),
            documento: "111.444.777-35".to_string(),
            email: "cliente@example.com".to_string(),
        })
        .await
        .expect("cadastro de cliente deveria funcionar");

    let cliente_uuid = uuid::Uuid::parse_str(&cliente.cliente_id).expect("uuid valido");
    let cliente_id = cadastro_api::domain::ClienteId::from_uuid(cliente_uuid);

    let found = env
        .get_cliente
        .execute(cliente_id)
        .await
        .expect("consulta deveria funcionar");
    assert_eq!(found.nome, "Rodrigo Furlaneti");

    let listed = env.list_clientes.execute().await.expect("listagem deveria funcionar");
    assert_eq!(listed.clientes.len(), 1);

    env.update_cliente
        .execute(
            cliente_id,
            UpdateClienteInput {
                nome: "Rodrigo F.".to_string(),
                documento: "111.444.777-35".to_string(),
                email: "cliente-novo@example.com".to_string(),
            },
        )
        .await
        .expect("atualizacao deveria funcionar");

    env.delete_cliente
        .execute(cliente_id)
        .await
        .expect("remocao deveria funcionar");

    let audit_log = read_audit_log(&env.audit_log_path).await;
    assert!(audit_log.contains("CLIENTE") && audit_log.contains("INSERT"));
    assert!(audit_log.contains("UPDATE") && audit_log.contains("cliente-novo@example.com"));
    assert!(audit_log.contains("DELETE"));
}

#[tokio::test]
async fn rejeita_cadastro_de_cliente_com_documento_invalido() {
    let env = build_test_env("documento_invalido");

    let result = env
        .register_cliente
        .execute(RegisterClienteInput {
            nome: "Fulano".to_string(),
            documento: "111.111.111-11".to_string(),
            email: "fulano@example.com".to_string(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn rejeita_cadastro_de_cliente_com_documento_duplicado() {
    let env = build_test_env("documento_duplicado");

    let input = || RegisterClienteInput {
        nome: "Fulano".to_string(),
        documento: "111.444.777-35".to_string(),
        email: "fulano@example.com".to_string(),
    };

    env.register_cliente
        .execute(input())
        .await
        .expect("primeiro cadastro deveria funcionar");
    let second_attempt = env.register_cliente.execute(input()).await;

    assert!(second_attempt.is_err());
}

#[tokio::test]
async fn delete_cliente_falha_para_cliente_inexistente() {
    let env = build_test_env("cliente_inexistente");
    let id_aleatorio = cadastro_api::domain::ClienteId::new();

    let result = env.delete_cliente.execute(id_aleatorio).await;

    assert!(result.is_err());
}

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cadastro_api::application::audit_logger::AuditLogger;
use cadastro_api::application::authenticate_user::AuthenticateUser;
use cadastro_api::application::delete_cliente::DeleteCliente;
use cadastro_api::application::delete_user::DeleteUser;
use cadastro_api::application::dto::{
    AuthenticateUserInput, RegisterClienteInput, RegisterUserInput, UpdateUserInput,
};
use cadastro_api::application::register_cliente::RegisterCliente;
use cadastro_api::application::register_user::RegisterUser;
use cadastro_api::application::update_user::UpdateUser;
use cadastro_api::domain::{ClienteRepository, UserRepository};
use cadastro_api::infra::audited_cliente_repository::AuditedClienteRepository;
use cadastro_api::infra::audited_user_repository::AuditedUserRepository;
use cadastro_api::infra::bcrypt_hasher::BcryptPasswordHasher;
use cadastro_api::infra::cache_cliente_repository::CacheClienteRepository;
use cadastro_api::infra::cache_user_repository::CacheUserRepository;
use cadastro_api::infra::file_audit_logger::FileAuditLogger;
use cadastro_api::infra::jwt_token_service::JwtTokenService;

/// Cobertura herdada da Semana 10 (usuario/cliente), adaptada ao novo
/// payload estruturado de auditoria. A cobertura de Produto/Pedido e da
/// reidratacao mora em `tests/produtos_pedidos_flow.rs`.
struct Ambiente {
    register_user: RegisterUser,
    authenticate_user: AuthenticateUser,
    update_user: UpdateUser,
    delete_user: DeleteUser,
    register_cliente: RegisterCliente,
    delete_cliente: DeleteCliente,
    audit_log_path: PathBuf,
}

fn montar_ambiente(nome_teste: &str) -> Ambiente {
    let audit_log_path =
        std::env::temp_dir().join(format!("auditoria_teste_{}_{}.txt", nome_teste, uuid::Uuid::new_v4()));
    let audit: Arc<dyn AuditLogger> = Arc::new(FileAuditLogger::start(audit_log_path.clone()));

    let usuarios: Arc<dyn UserRepository> = Arc::new(AuditedUserRepository::new(
        Arc::new(CacheUserRepository::new()),
        audit.clone(),
    ));
    let clientes: Arc<dyn ClienteRepository> = Arc::new(AuditedClienteRepository::new(
        Arc::new(CacheClienteRepository::new()),
        audit,
    ));

    let hasher = Arc::new(BcryptPasswordHasher::new(4));
    let tokens = Arc::new(JwtTokenService::new("test-secret".to_string(), 3600));

    Ambiente {
        register_user: RegisterUser::new(usuarios.clone(), hasher.clone()),
        authenticate_user: AuthenticateUser::new(usuarios.clone(), hasher, tokens),
        update_user: UpdateUser::new(usuarios.clone()),
        delete_user: DeleteUser::new(usuarios),
        register_cliente: RegisterCliente::new(clientes.clone()),
        delete_cliente: DeleteCliente::new(clientes),
        audit_log_path,
    }
}

async fn ler_auditoria(path: &PathBuf) -> String {
    tokio::time::sleep(Duration::from_millis(100)).await;
    tokio::fs::read_to_string(path).await.unwrap_or_default()
}

#[tokio::test]
async fn registra_autentica_atualiza_e_remove_usuario_gerando_auditoria() {
    let ambiente = montar_ambiente("usuario_crud");

    let registro = ambiente
        .register_user
        .execute(RegisterUserInput {
            email: "rodrigo@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let login = ambiente
        .authenticate_user
        .execute(AuthenticateUserInput {
            email: "rodrigo@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("login deveria funcionar");
    assert!(!login.access_token.is_empty());

    let user_id = cadastro_api::domain::UserId::from_uuid(uuid::Uuid::parse_str(&registro.user_id).unwrap());

    ambiente
        .update_user
        .execute(
            user_id,
            UpdateUserInput {
                email: "rodrigo.novo@example.com".to_string(),
            },
        )
        .await
        .expect("atualizacao deveria funcionar");

    ambiente.delete_user.execute(user_id).await.expect("remocao deveria funcionar");

    let auditoria = ler_auditoria(&ambiente.audit_log_path).await;
    assert!(auditoria.contains("\"Usuario\"") && auditoria.contains("\"Insert\""));
    assert!(auditoria.contains("\"Update\"") && auditoria.contains("rodrigo.novo@example.com"));
    assert!(auditoria.contains("\"Delete\""));
}

#[tokio::test]
async fn cadastra_e_remove_cliente_gerando_auditoria() {
    let ambiente = montar_ambiente("cliente_crud");

    let cliente = ambiente
        .register_cliente
        .execute(RegisterClienteInput {
            nome: "Rodrigo Furlaneti".to_string(),
            documento: "111.444.777-35".to_string(),
            email: "cliente@example.com".to_string(),
        })
        .await
        .expect("cadastro deveria funcionar");

    let cliente_id =
        cadastro_api::domain::ClienteId::from_uuid(uuid::Uuid::parse_str(&cliente.cliente_id).unwrap());
    ambiente
        .delete_cliente
        .execute(cliente_id)
        .await
        .expect("remocao deveria funcionar");

    let auditoria = ler_auditoria(&ambiente.audit_log_path).await;
    assert!(auditoria.contains("\"Cliente\"") && auditoria.contains("\"Insert\""));
    assert!(auditoria.contains("\"Delete\""));
}

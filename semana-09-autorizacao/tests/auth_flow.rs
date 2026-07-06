use std::sync::Arc;

use auth_api::application::authenticate_user::AuthenticateUser;
use auth_api::application::dto::{AuthenticateUserInput, RegisterUserInput};
use auth_api::application::get_current_user::GetCurrentUser;
use auth_api::application::register_user::RegisterUser;
use auth_api::application::token_service::TokenService;
use auth_api::domain::{Role, UserRepository};
use auth_api::infra::bcrypt_hasher::BcryptPasswordHasher;
use auth_api::infra::db::create_in_memory_sqlite_pool;
use auth_api::infra::jwt_token_service::JwtTokenService;
use auth_api::infra::sqlite_user_repository::SqliteUserRepository;

/// Cada teste monta seu proprio banco SQLite em memoria (isolado dos
/// outros testes) e liga os casos de uso reais na frente dele. Nenhum
/// mock de dominio/aplicacao e necessario: e a mesma composicao usada em
/// producao, so trocando Postgres por SQLite em memoria.
struct UseCases {
    register_user: RegisterUser,
    authenticate_user: AuthenticateUser,
    get_current_user: GetCurrentUser,
    tokens: Arc<dyn TokenService>,
}

async fn build_use_cases() -> UseCases {
    let pool = create_in_memory_sqlite_pool().await;
    let repository: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(pool));
    let hasher = Arc::new(BcryptPasswordHasher::new(4)); // custo baixo: testes rapidos
    let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new("test-secret".to_string(), 3600));

    let register_user = RegisterUser::new(repository.clone(), hasher.clone());
    let authenticate_user =
        AuthenticateUser::new(repository.clone(), hasher, tokens.clone());
    let get_current_user = GetCurrentUser::new(repository);

    UseCases {
        register_user,
        authenticate_user,
        get_current_user,
        tokens,
    }
}

#[tokio::test]
async fn registra_e_autentica_um_usuario_com_sucesso() {
    let uc = build_use_cases().await;

    uc.register_user
        .execute(RegisterUserInput {
            email: "teste@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let output = uc
        .authenticate_user
        .execute(AuthenticateUserInput {
            email: "teste@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("login deveria funcionar com a senha correta");

    assert!(!output.access_token.is_empty());
}

#[tokio::test]
async fn rejeita_login_com_senha_errada() {
    let uc = build_use_cases().await;

    uc.register_user
        .execute(RegisterUserInput {
            email: "teste2@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let result = uc
        .authenticate_user
        .execute(AuthenticateUserInput {
            email: "teste2@example.com".to_string(),
            password: "SenhaErrada999".to_string(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn rejeita_cadastro_de_email_duplicado() {
    let uc = build_use_cases().await;

    let input = || RegisterUserInput {
        email: "duplicado@example.com".to_string(),
        password: "SenhaForte123".to_string(),
    };

    uc.register_user
        .execute(input())
        .await
        .expect("primeiro cadastro deveria funcionar");
    let second_attempt = uc.register_user.execute(input()).await;

    assert!(second_attempt.is_err());
}

#[tokio::test]
async fn usuario_registrado_publicamente_nunca_recebe_role_admin() {
    let uc = build_use_cases().await;

    let register_output = uc
        .register_user
        .execute(RegisterUserInput {
            email: "usuario-comum@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    // Regra de negocio central da Semana 9: nao existe parametro "role" no
    // endpoint publico de cadastro, entao e estruturalmente impossivel
    // qualquer requisicao externa criar um admin por esse caminho.
    assert_eq!(register_output.role, "user");
}

#[tokio::test]
async fn token_emitido_no_login_carrega_a_role_correta_e_bate_no_verify() {
    let uc = build_use_cases().await;

    uc.register_user
        .execute(RegisterUserInput {
            email: "checagem-role@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let login_output = uc
        .authenticate_user
        .execute(AuthenticateUserInput {
            email: "checagem-role@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("login deveria funcionar");

    let claims = uc
        .tokens
        .verify(&login_output.access_token)
        .expect("token emitido deveria ser valido");

    assert_eq!(claims.role, Role::User);
}

#[tokio::test]
async fn get_current_user_retorna_os_dados_de_quem_esta_logado() {
    let uc = build_use_cases().await;

    let register_output = uc
        .register_user
        .execute(RegisterUserInput {
            email: "meu-perfil@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let login_output = uc
        .authenticate_user
        .execute(AuthenticateUserInput {
            email: "meu-perfil@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("login deveria funcionar");

    let claims = uc
        .tokens
        .verify(&login_output.access_token)
        .expect("token deveria ser valido");

    let profile = uc
        .get_current_user
        .execute(claims.user_id)
        .await
        .expect("busca do usuario logado deveria funcionar");

    assert_eq!(profile.user_id, register_output.user_id);
    assert_eq!(profile.email, "meu-perfil@example.com");
    assert_eq!(profile.role, "user");
}

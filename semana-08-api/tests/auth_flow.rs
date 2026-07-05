use std::sync::Arc;

use auth_api::application::authenticate_user::AuthenticateUser;
use auth_api::application::dto::{AuthenticateUserInput, RegisterUserInput};
use auth_api::application::register_user::RegisterUser;
use auth_api::domain::UserRepository;
use auth_api::infra::bcrypt_hasher::BcryptPasswordHasher;
use auth_api::infra::db::create_in_memory_sqlite_pool;
use auth_api::infra::jwt_token_service::JwtTokenService;
use auth_api::infra::sqlite_user_repository::SqliteUserRepository;

/// Cada teste monta seu proprio banco SQLite em memoria (isolado dos
/// outros testes) e liga os casos de uso reais na frente dele. Nenhum
/// mock de dominio/aplicacao e necessario: e a mesma composicao usada em
/// produtos, so trocando Postgres por SQLite em memoria.
async fn build_use_cases_async() -> (RegisterUser, AuthenticateUser) {
    let pool = create_in_memory_sqlite_pool().await;
    let repository: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(pool));
    let hasher = Arc::new(BcryptPasswordHasher::new(4)); // custo baixo: testes rapidos
    let tokens = Arc::new(JwtTokenService::new("test-secret".to_string(), 3600));

    let register_user = RegisterUser::new(repository.clone(), hasher.clone());
    let authenticate_user = AuthenticateUser::new(repository, hasher, tokens);

    (register_user, authenticate_user)
}

#[tokio::test]
async fn registra_e_autentica_um_usuario_com_sucesso() {
    let (register_user, authenticate_user) = build_use_cases_async().await;

    register_user
        .execute(RegisterUserInput {
            email: "teste@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let output = authenticate_user
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
    let (register_user, authenticate_user) = build_use_cases_async().await;

    register_user
        .execute(RegisterUserInput {
            email: "teste2@example.com".to_string(),
            password: "SenhaForte123".to_string(),
        })
        .await
        .expect("registro deveria funcionar");

    let result = authenticate_user
        .execute(AuthenticateUserInput {
            email: "teste2@example.com".to_string(),
            password: "SenhaErrada999".to_string(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn rejeita_cadastro_de_email_duplicado() {
    let (register_user, _authenticate_user) = build_use_cases_async().await;

    let input = || RegisterUserInput {
        email: "duplicado@example.com".to_string(),
        password: "SenhaForte123".to_string(),
    };

    register_user.execute(input()).await.expect("primeiro cadastro deveria funcionar");
    let second_attempt = register_user.execute(input()).await;

    assert!(second_attempt.is_err());
}

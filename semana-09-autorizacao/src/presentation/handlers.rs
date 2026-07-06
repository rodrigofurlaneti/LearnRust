use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::application::dto::{
    AuthenticateUserInput, AuthenticateUserOutput, CurrentUserOutput, RegisterUserInput,
    RegisterUserOutput,
};
use crate::presentation::auth_extractor::{AdminUser, AuthenticatedUser};
use crate::presentation::error_response::{ApiError, ErrorBody};
use crate::presentation::state::AppState;

// Handlers sao "burros" de proposito: so traduzem HTTP <-> caso de uso.
// Nenhuma regra de negocio mora aqui (SRP). As anotacoes `#[utoipa::path]`
// sao metadados de documentacao lidos em tempo de compilacao - nao mudam
// o comportamento em runtime. (Comentario normal `//`, de proposito: um
// `///` aqui viraria a descricao publica do endpoint no Swagger.)
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    summary = "Cadastra um novo usuario",
    request_body = RegisterUserInput,
    responses(
        (status = 200, description = "Usuario criado com sucesso", body = RegisterUserOutput),
        (status = 400, description = "Email invalido ou senha fraca", body = ErrorBody),
        (status = 409, description = "Ja existe usuario com este email", body = ErrorBody),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterUserInput>,
) -> Result<Json<RegisterUserOutput>, ApiError> {
    let output = state.register_user.execute(input).await?;
    Ok(Json(output))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    summary = "Autentica um usuario e emite um access token",
    request_body = AuthenticateUserInput,
    responses(
        (status = 200, description = "Login efetuado, retorna o access token", body = AuthenticateUserOutput),
        (status = 400, description = "Email invalido ou senha fraca", body = ErrorBody),
        (status = 401, description = "Credenciais invalidas", body = ErrorBody),
        (status = 429, description = "Muitas tentativas em pouco tempo", body = ErrorBody),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<AuthenticateUserInput>,
) -> Result<Json<AuthenticateUserOutput>, ApiError> {
    let output = state.authenticate_user.execute(input).await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    summary = "Dados do usuario autenticado",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Usuario autenticado", body = CurrentUserOutput),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
    )
)]
pub async fn me(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<Json<CurrentUserOutput>, ApiError> {
    let output = state.get_current_user.execute(authenticated.user_id).await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/admin/ping",
    tag = "admin",
    summary = "Endpoint de exemplo restrito a administradores (RBAC)",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Voce esta autenticado e tem papel admin"),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 403, description = "Autenticado, mas sem permissao de admin", body = ErrorBody),
    )
)]
pub async fn admin_ping(AdminUser(admin): AdminUser) -> Json<Value> {
    Json(json!({
        "message": "pong - voce e admin",
        "user_id": admin.user_id.as_uuid().to_string(),
    }))
}

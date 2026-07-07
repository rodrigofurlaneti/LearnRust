use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::application::dto::{UpdateUserInput, UpdateUserOutput};
use crate::domain::UserId;
use crate::presentation::auth_extractor::AdminUser;
use crate::presentation::error_response::{ApiError, ErrorBody};
use crate::presentation::path_id::parse_uuid_path;
use crate::presentation::state::AppState;

// Handlers novos na Semana 10: gestao administrativa de usuarios. Ambos
// exigem `AdminUser` (mesmo guard de RBAC usado em `admin_ping` na
// Semana 9) - nenhum usuario comum pode editar/remover outra conta.
#[utoipa::path(
    put,
    path = "/admin/usuarios/{id}",
    tag = "admin",
    summary = "Atualiza o cadastro de um usuario (somente admin)",
    security(("bearer_auth" = [])),
    request_body = UpdateUserInput,
    responses(
        (status = 200, description = "Usuario atualizado", body = UpdateUserOutput),
        (status = 400, description = "Email invalido ou id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 403, description = "Autenticado, mas sem permissao de admin", body = ErrorBody),
        (status = 404, description = "Usuario nao encontrado", body = ErrorBody),
        (status = 409, description = "Ja existe outro usuario com este email", body = ErrorBody),
    )
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AdminUser(_admin): AdminUser,
    Json(input): Json<UpdateUserInput>,
) -> Result<Json<UpdateUserOutput>, ApiError> {
    let user_id = UserId::from_uuid(parse_uuid_path(&id)?);
    let output = state.update_user.execute(user_id, input).await?;
    Ok(Json(output))
}

#[utoipa::path(
    delete,
    path = "/admin/usuarios/{id}",
    tag = "admin",
    summary = "Remove um usuario (somente admin)",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Usuario removido"),
        (status = 400, description = "Id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 403, description = "Autenticado, mas sem permissao de admin", body = ErrorBody),
        (status = 404, description = "Usuario nao encontrado", body = ErrorBody),
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AdminUser(_admin): AdminUser,
) -> Result<StatusCode, ApiError> {
    let user_id = UserId::from_uuid(parse_uuid_path(&id)?);
    state.delete_user.execute(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

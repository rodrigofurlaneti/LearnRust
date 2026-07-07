use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::application::cliente::commands::{
    DeleteClienteCommand, RegisterClienteCommand, UpdateClienteCommand,
};
use crate::application::cliente::queries::{GetClienteQuery, ListClientesQuery};
use crate::application::shared::dto::{
    ClienteOutput, ListClientesOutput, RegisterClienteInput, UpdateClienteInput,
};
use crate::domain::ClienteId;
use crate::presentation::shared::auth_extractor::{AdminUser, AuthenticatedUser};
use crate::presentation::shared::error_response::{ApiError, ErrorBody};
use crate::presentation::shared::path_id::parse_uuid_path;
use crate::presentation::shared::state::AppState;

// Cadastro de cliente, novo na Semana 10. Criacao/consulta/edicao exigem
// so `AuthenticatedUser` (qualquer usuario logado atende clientes); a
// remocao exige `AdminUser`. Semana 13: os handlers deixam de chamar um
// caso de uso especifico e passam a despachar um Command/Query pelo
// barramento do `AppState`.
#[utoipa::path(
    post,
    path = "/clientes",
    tag = "clientes",
    summary = "Cadastra um novo cliente",
    security(("bearer_auth" = [])),
    request_body = RegisterClienteInput,
    responses(
        (status = 200, description = "Cliente criado com sucesso", body = ClienteOutput),
        (status = 400, description = "Nome, documento ou email invalidos", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 409, description = "Ja existe cliente com este documento", body = ErrorBody),
    )
)]
pub async fn create_cliente(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(input): Json<RegisterClienteInput>,
) -> Result<Json<ClienteOutput>, ApiError> {
    let output = state
        .command_bus
        .dispatch(RegisterClienteCommand(input))
        .await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/clientes/{id}",
    tag = "clientes",
    summary = "Consulta um cliente pelo id",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Cliente encontrado", body = ClienteOutput),
        (status = 400, description = "Id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 404, description = "Cliente nao encontrado", body = ErrorBody),
    )
)]
pub async fn get_cliente(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _authenticated: AuthenticatedUser,
) -> Result<Json<ClienteOutput>, ApiError> {
    let cliente_id = ClienteId::from_uuid(parse_uuid_path(&id)?);
    let output = state
        .query_bus
        .dispatch(GetClienteQuery { cliente_id })
        .await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/clientes",
    tag = "clientes",
    summary = "Lista todos os clientes cadastrados",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Lista de clientes", body = ListClientesOutput),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
    )
)]
pub async fn list_clientes(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
) -> Result<Json<ListClientesOutput>, ApiError> {
    let output = state.query_bus.dispatch(ListClientesQuery).await?;
    Ok(Json(output))
}

#[utoipa::path(
    put,
    path = "/clientes/{id}",
    tag = "clientes",
    summary = "Atualiza o cadastro de um cliente",
    security(("bearer_auth" = [])),
    request_body = UpdateClienteInput,
    responses(
        (status = 200, description = "Cliente atualizado", body = ClienteOutput),
        (status = 400, description = "Nome, documento, email ou id invalidos", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 404, description = "Cliente nao encontrado", body = ErrorBody),
        (status = 409, description = "Ja existe outro cliente com este documento", body = ErrorBody),
    )
)]
pub async fn update_cliente(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _authenticated: AuthenticatedUser,
    Json(input): Json<UpdateClienteInput>,
) -> Result<Json<ClienteOutput>, ApiError> {
    let cliente_id = ClienteId::from_uuid(parse_uuid_path(&id)?);
    let output = state
        .command_bus
        .dispatch(UpdateClienteCommand { cliente_id, input })
        .await?;
    Ok(Json(output))
}

#[utoipa::path(
    delete,
    path = "/clientes/{id}",
    tag = "clientes",
    summary = "Remove um cliente (somente admin)",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Cliente removido"),
        (status = 400, description = "Id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 403, description = "Autenticado, mas sem permissao de admin", body = ErrorBody),
        (status = 404, description = "Cliente nao encontrado", body = ErrorBody),
    )
)]
pub async fn delete_cliente(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AdminUser(_admin): AdminUser,
) -> Result<StatusCode, ApiError> {
    let cliente_id = ClienteId::from_uuid(parse_uuid_path(&id)?);
    state
        .command_bus
        .dispatch(DeleteClienteCommand { cliente_id })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

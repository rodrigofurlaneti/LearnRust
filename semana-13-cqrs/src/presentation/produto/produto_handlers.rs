use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::application::produto::commands::{
    DeleteProdutoCommand, RegisterProdutoCommand, UpdateProdutoCommand,
};
use crate::application::produto::queries::{GetProdutoQuery, ListProdutosQuery};
use crate::application::shared::dto::{
    ListProdutosOutput, ProdutoOutput, RegisterProdutoInput, UpdateProdutoInput,
};
use crate::domain::ProdutoId;
use crate::presentation::shared::auth_extractor::{AdminUser, AuthenticatedUser};
use crate::presentation::shared::error_response::{ApiError, ErrorBody};
use crate::presentation::shared::path_id::parse_uuid_path;
use crate::presentation::shared::state::AppState;

// Cadastro de produto, novo na Semana 11. Mesma politica de protecao do
// cadastro de cliente: criacao/consulta/edicao exigem so
// `AuthenticatedUser`; a remocao exige `AdminUser`. Semana 13: handlers
// despacham Command/Query pelo barramento em vez de chamar o caso de uso
// diretamente.
#[utoipa::path(
    post,
    path = "/produtos",
    tag = "produtos",
    summary = "Cadastra um novo produto",
    security(("bearer_auth" = [])),
    request_body = RegisterProdutoInput,
    responses(
        (status = 200, description = "Produto criado com sucesso", body = ProdutoOutput),
        (status = 400, description = "Nome ou preco invalidos", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
    )
)]
pub async fn create_produto(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(input): Json<RegisterProdutoInput>,
) -> Result<Json<ProdutoOutput>, ApiError> {
    let output = state
        .command_bus
        .dispatch(RegisterProdutoCommand(input))
        .await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/produtos/{id}",
    tag = "produtos",
    summary = "Consulta um produto pelo id",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Produto encontrado", body = ProdutoOutput),
        (status = 400, description = "Id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 404, description = "Produto nao encontrado", body = ErrorBody),
    )
)]
pub async fn get_produto(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _authenticated: AuthenticatedUser,
) -> Result<Json<ProdutoOutput>, ApiError> {
    let produto_id = ProdutoId::from_uuid(parse_uuid_path(&id)?);
    let output = state
        .query_bus
        .dispatch(GetProdutoQuery { produto_id })
        .await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/produtos",
    tag = "produtos",
    summary = "Lista todos os produtos cadastrados",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Lista de produtos", body = ListProdutosOutput),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
    )
)]
pub async fn list_produtos(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
) -> Result<Json<ListProdutosOutput>, ApiError> {
    let output = state.query_bus.dispatch(ListProdutosQuery).await?;
    Ok(Json(output))
}

#[utoipa::path(
    put,
    path = "/produtos/{id}",
    tag = "produtos",
    summary = "Atualiza o cadastro de um produto",
    security(("bearer_auth" = [])),
    request_body = UpdateProdutoInput,
    responses(
        (status = 200, description = "Produto atualizado", body = ProdutoOutput),
        (status = 400, description = "Nome, preco ou id invalidos", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 404, description = "Produto nao encontrado", body = ErrorBody),
    )
)]
pub async fn update_produto(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _authenticated: AuthenticatedUser,
    Json(input): Json<UpdateProdutoInput>,
) -> Result<Json<ProdutoOutput>, ApiError> {
    let produto_id = ProdutoId::from_uuid(parse_uuid_path(&id)?);
    let output = state
        .command_bus
        .dispatch(UpdateProdutoCommand { produto_id, input })
        .await?;
    Ok(Json(output))
}

#[utoipa::path(
    delete,
    path = "/produtos/{id}",
    tag = "produtos",
    summary = "Remove um produto (somente admin)",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Produto removido"),
        (status = 400, description = "Id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 403, description = "Autenticado, mas sem permissao de admin", body = ErrorBody),
        (status = 404, description = "Produto nao encontrado", body = ErrorBody),
    )
)]
pub async fn delete_produto(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AdminUser(_admin): AdminUser,
) -> Result<StatusCode, ApiError> {
    let produto_id = ProdutoId::from_uuid(parse_uuid_path(&id)?);
    state
        .command_bus
        .dispatch(DeleteProdutoCommand { produto_id })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::application::dto::{ListPedidosOutput, PedidoOutput, RegisterPedidoInput};
use crate::domain::PedidoId;
use crate::presentation::auth_extractor::{AdminUser, AuthenticatedUser};
use crate::presentation::error_response::{ApiError, ErrorBody};
use crate::presentation::path_id::parse_uuid_path;
use crate::presentation::state::AppState;

// Cadastro de pedido, novo na Semana 11. Sem rota de PUT de proposito
// (ver domain::pedido_repository) - um pedido errado e cancelado
// (DELETE, restrito a AdminUser) e refeito.
#[utoipa::path(
    post,
    path = "/pedidos",
    tag = "pedidos",
    summary = "Cria um pedido (um ou mais itens); congela preco/nome do produto no momento da compra",
    security(("bearer_auth" = [])),
    request_body = RegisterPedidoInput,
    responses(
        (status = 200, description = "Pedido criado com sucesso", body = PedidoOutput),
        (status = 400, description = "Cliente/produto/id invalidos, quantidade invalida ou pedido sem itens", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 404, description = "Cliente ou produto informado nao encontrado", body = ErrorBody),
    )
)]
pub async fn create_pedido(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(input): Json<RegisterPedidoInput>,
) -> Result<Json<PedidoOutput>, ApiError> {
    let output = state.register_pedido.execute(input).await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/pedidos/{id}",
    tag = "pedidos",
    summary = "Consulta um pedido pelo id",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pedido encontrado", body = PedidoOutput),
        (status = 400, description = "Id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 404, description = "Pedido nao encontrado", body = ErrorBody),
    )
)]
pub async fn get_pedido(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _authenticated: AuthenticatedUser,
) -> Result<Json<PedidoOutput>, ApiError> {
    let pedido_id = PedidoId::from_uuid(parse_uuid_path(&id)?);
    let output = state.get_pedido.execute(pedido_id).await?;
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/pedidos",
    tag = "pedidos",
    summary = "Lista todos os pedidos cadastrados",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Lista de pedidos", body = ListPedidosOutput),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
    )
)]
pub async fn list_pedidos(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
) -> Result<Json<ListPedidosOutput>, ApiError> {
    let output = state.list_pedidos.execute().await?;
    Ok(Json(output))
}

#[utoipa::path(
    delete,
    path = "/pedidos/{id}",
    tag = "pedidos",
    summary = "Cancela (remove) um pedido (somente admin)",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Pedido removido"),
        (status = 400, description = "Id malformado", body = ErrorBody),
        (status = 401, description = "Token ausente, invalido ou expirado", body = ErrorBody),
        (status = 403, description = "Autenticado, mas sem permissao de admin", body = ErrorBody),
        (status = 404, description = "Pedido nao encontrado", body = ErrorBody),
    )
)]
pub async fn delete_pedido(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AdminUser(_admin): AdminUser,
) -> Result<StatusCode, ApiError> {
    let pedido_id = PedidoId::from_uuid(parse_uuid_path(&id)?);
    state.delete_pedido.execute(pedido_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

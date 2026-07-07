use std::sync::Arc;

use axum::routing::{get, post, put};
use axum::Router;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::GovernorConfig;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use tower_governor::GovernorLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::presentation::handlers::{
    admin_ping, create_cliente, delete_cliente, delete_user, get_cliente, list_clientes, login,
    me, register, update_cliente, update_user,
};
use crate::presentation::openapi::ApiDoc;
use crate::presentation::state::AppState;

/// Limite de tentativas de login: no maximo 1 requisicao por segundo em
/// media, com rajada (`burst`) de ate 5. Herdado sem alteracoes da
/// Semana 9 - so essa rota tem essa camada extra.
fn login_rate_limit_layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let config: Arc<GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware>> = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(5)
            .finish()
            .expect("configuracao de rate limit invalida"),
    );

    GovernorLayer { config }
}

pub fn build_router(state: AppState) -> Router {
    let login_route = Router::new()
        .route("/auth/login", post(login))
        .layer(login_rate_limit_layer())
        .with_state(state.clone());

    let auth_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/me", get(me));

    let admin_routes = Router::new()
        .route("/admin/ping", get(admin_ping))
        .route(
            "/admin/usuarios/:id",
            put(update_user).delete(delete_user),
        );

    let cliente_routes = Router::new()
        .route("/clientes", post(create_cliente).get(list_clientes))
        .route(
            "/clientes/:id",
            get(get_cliente).put(update_cliente).delete(delete_cliente),
        );

    let other_routes = auth_routes
        .merge(admin_routes)
        .merge(cliente_routes)
        .with_state(state);

    other_routes
        .merge(login_route)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

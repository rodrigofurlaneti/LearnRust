use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::governor::GovernorConfig;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use tower_governor::GovernorLayer;
use governor::middleware::NoOpMiddleware;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::presentation::handlers::{admin_ping, login, me, register};
use crate::presentation::openapi::ApiDoc;
use crate::presentation::state::AppState;

/// Limite de tentativas de login: no maximo 1 requisicao por segundo em
/// media, com rajada (`burst`) de ate 5 - o suficiente para um usuario
/// real errar a senha algumas vezes, mas nao para um ataque de forca
/// bruta. So essa rota tem essa camada extra: registro e leitura de perfil
/// nao precisam do mesmo nivel de protecao.
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

    let other_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/me", get(me))
        .route("/admin/ping", get(admin_ping))
        .with_state(state);

    other_routes
        .merge(login_route)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

use axum::routing::post;
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::presentation::handlers::{login, register};
use crate::presentation::openapi::ApiDoc;
use crate::presentation::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .with_state(state);

    api_routes.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

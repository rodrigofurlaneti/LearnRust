use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::application::dto::{
    AuthenticateUserInput, AuthenticateUserOutput, CurrentUserOutput, RegisterUserInput,
    RegisterUserOutput,
};
use crate::presentation::error_response::ErrorBody;
use crate::presentation::handlers;

/// Ponto unico que agrega toda a documentacao OpenAPI da API. So existe na
/// camada de presentation - domain/application nao sabem que Swagger existe.
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::register,
        handlers::login,
        handlers::me,
        handlers::admin_ping
    ),
    components(schemas(
        RegisterUserInput,
        RegisterUserOutput,
        AuthenticateUserInput,
        AuthenticateUserOutput,
        CurrentUserOutput,
        ErrorBody
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Registro e autenticacao de usuarios"),
        (name = "admin", description = "Exemplo de rota restrita por RBAC (Role::Admin)")
    )
)]
pub struct ApiDoc;

/// Ensina o Swagger UI a mostrar o botao "Authorize" e mandar o header
/// `Authorization: Bearer <token>` nas rotas marcadas com
/// `security(("bearer_auth" = []))`.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

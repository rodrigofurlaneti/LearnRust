use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::application::dto::{
    AuthenticateUserInput, AuthenticateUserOutput, ClienteOutput, CurrentUserOutput,
    ListClientesOutput, RegisterClienteInput, RegisterUserInput, RegisterUserOutput,
    UpdateClienteInput, UpdateUserInput, UpdateUserOutput,
};
use crate::presentation::error_response::ErrorBody;
use crate::presentation::handlers::{auth_handlers, cliente_handlers, user_handlers};

/// Ponto unico que agrega toda a documentacao OpenAPI da API. So existe na
/// camada de presentation - domain/application nao sabem que Swagger
/// existe. Ampliado nesta semana com as rotas de gestao de usuario e o
/// CRUD de cliente.
///
/// Nota tecnica: cada utoipa::path gera, ao lado da funcao do handler, um
/// item auxiliar (prefixo __path_) no MESMO modulo - por isso os handlers
/// precisam ser referenciados aqui pelo caminho do modulo onde foram
/// declarados (auth_handlers::register, e nao so handlers::register); um
/// pub use em handlers/mod.rs reexporta a funcao, mas nao o item
/// auxiliar gerado pela macro.
#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handlers::register,
        auth_handlers::login,
        auth_handlers::me,
        auth_handlers::admin_ping,
        user_handlers::update_user,
        user_handlers::delete_user,
        cliente_handlers::create_cliente,
        cliente_handlers::get_cliente,
        cliente_handlers::list_clientes,
        cliente_handlers::update_cliente,
        cliente_handlers::delete_cliente
    ),
    components(schemas(
        RegisterUserInput,
        RegisterUserOutput,
        AuthenticateUserInput,
        AuthenticateUserOutput,
        CurrentUserOutput,
        UpdateUserInput,
        UpdateUserOutput,
        RegisterClienteInput,
        UpdateClienteInput,
        ClienteOutput,
        ListClientesOutput,
        ErrorBody
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Registro e autenticacao de usuarios"),
        (name = "admin", description = "Gestao administrativa de usuarios e exemplo de RBAC (Role::Admin)"),
        (name = "clientes", description = "Cadastro de clientes (Semana 10), armazenado em cache")
    )
)]
pub struct ApiDoc;

// Ensina o Swagger UI a mostrar o botao "Authorize" e mandar o header
// Authorization: Bearer <token> nas rotas marcadas com
// security(("bearer_auth" = [])).
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

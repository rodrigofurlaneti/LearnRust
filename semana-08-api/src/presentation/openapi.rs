use utoipa::OpenApi;

use crate::application::dto::{
    AuthenticateUserInput, AuthenticateUserOutput, RegisterUserInput, RegisterUserOutput,
};
use crate::presentation::error_response::ErrorBody;
use crate::presentation::handlers;

/// Ponto unico que agrega toda a documentacao OpenAPI da API. So existe na
/// camada de presentation - domain/application nao sabem que Swagger existe.
#[derive(OpenApi)]
#[openapi(
    paths(handlers::register, handlers::login),
    components(schemas(
        RegisterUserInput,
        RegisterUserOutput,
        AuthenticateUserInput,
        AuthenticateUserOutput,
        ErrorBody
    )),
    tags(
        (name = "auth", description = "Registro e autenticacao de usuarios")
    )
)]
pub struct ApiDoc;

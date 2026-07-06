use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// DTOs de entrada/saida dos casos de uso. Ficam na aplicacao (nao na
/// presentation) porque representam a "linguagem" dos casos de uso, e a
/// presentation apenas (de)serializa isso de/para JSON.
///
/// Trade-off consciente: `ToSchema` (documentacao OpenAPI/Swagger) e
/// derivado aqui, junto com `Serialize`/`Deserialize`. O ideal em Clean
/// Architecture "estrito" seria a presentation ter suas proprias structs
/// de schema, mas isso duplicaria os campos e arriscaria a documentacao
/// ficar dessincronizada do formato real. Preferimos aceitar essa pequena
/// dependencia de `application` em `utoipa` (mesma categoria de `serde`,
/// ja aceita) a correr esse risco.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterUserInput {
    #[schema(example = "rodrigo@example.com")]
    pub email: String,
    #[schema(example = "SenhaForte123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterUserOutput {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthenticateUserInput {
    #[schema(example = "rodrigo@example.com")]
    pub email: String,
    #[schema(example = "SenhaForte123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthenticateUserOutput {
    pub access_token: String,
}

/// Saida de `GET /me` - dados do usuario associado ao token enviado.
#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserOutput {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

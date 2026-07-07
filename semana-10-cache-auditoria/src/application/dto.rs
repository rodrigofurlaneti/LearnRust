use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// DTOs de entrada/saida dos casos de uso. Ficam na aplicacao (nao na
/// presentation) porque representam a "linguagem" dos casos de uso, e a
/// presentation apenas (de)serializa isso de/para JSON. Mesmo trade-off
/// documentado na Semana 9: `ToSchema` (Swagger) e derivado aqui junto
/// com `Serialize`/`Deserialize`.

// ---------------------------------------------------------------------
// Usuario (auth) - herdado da Semana 9, com UpdateUser/DeleteUser novos.
// ---------------------------------------------------------------------

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

/// Entrada de `PUT /admin/usuarios/:id` (novo na Semana 10, restrito a
/// `AdminUser`). O id vem da URL, nao do corpo - so o email e editavel
/// por este caso de uso.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserInput {
    #[schema(example = "novo-email@example.com")]
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateUserOutput {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

// ---------------------------------------------------------------------
// Cliente (novo na Semana 10).
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterClienteInput {
    #[schema(example = "Rodrigo Furlaneti")]
    pub nome: String,
    #[schema(example = "111.444.777-35")]
    pub documento: String,
    #[schema(example = "cliente@example.com")]
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateClienteInput {
    #[schema(example = "Rodrigo Furlaneti")]
    pub nome: String,
    #[schema(example = "111.444.777-35")]
    pub documento: String,
    #[schema(example = "cliente@example.com")]
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClienteOutput {
    pub cliente_id: String,
    pub nome: String,
    pub documento: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListClientesOutput {
    pub clientes: Vec<ClienteOutput>,
}

//! O que atravessa todos os agregados na camada de apresentacao: o
//! extractor de autenticacao/autorizacao, o mapeamento de erro para status
//! HTTP, o Swagger/OpenAPI, o parser de `:id` de rota, a montagem do
//! `Router` e o `AppState` compartilhado.

pub mod auth_extractor;
pub mod error_response;
pub mod openapi;
pub mod path_id;
pub mod routes;
pub mod state;

pub mod authenticate_user;
pub mod dto;
pub mod errors;
pub mod register_user;
pub mod token_service;

// Reexportados para quem for consumir esta camada de fora (ex.: presentation
// ou testes de integracao) sem precisar conhecer os modulos internos.
#[allow(unused_imports)]
pub use errors::ApplicationError;
#[allow(unused_imports)]
pub use token_service::{AccessToken, TokenService};

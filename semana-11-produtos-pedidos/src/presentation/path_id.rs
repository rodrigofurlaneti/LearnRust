use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::DomainError;
use crate::presentation::error_response::ApiError;

/// Converte o segmento `:id` de uma rota (sempre uma `String` crua vinda
/// do Axum) num `Uuid`, ou devolve 400 se o formato for invalido. Um unico
/// lugar para essa conversao evita repetir `Uuid::parse_str(...).map_err`
/// em cada handler que recebe um id na URL.
pub fn parse_uuid_path(raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw)
        .map_err(|_| ApiError::from(ApplicationError::Domain(DomainError::InvalidId)))
}

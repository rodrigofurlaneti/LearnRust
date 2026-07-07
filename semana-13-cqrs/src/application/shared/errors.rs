use thiserror::Error;

use crate::domain::DomainError;

/// Erros da camada de aplicacao: ou repassam uma violacao de regra de
/// negocio do dominio, ou representam uma falha tecnica inesperada
/// (infra fora do ar, etc).
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("erro interno: {0}")]
    Unexpected(String),
}

use thiserror::Error;

use crate::domain::DomainError;

/// Erros da camada de aplicação: ou repassam uma violação de regra de
/// negócio do domínio, ou representam uma falha técnica inesperada
/// (infra fora do ar, etc).
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("erro interno: {0}")]
    Unexpected(String),
}

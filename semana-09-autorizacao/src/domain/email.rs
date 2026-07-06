use once_cell::sync::Lazy;
use regex::Regex;

use crate::domain::errors::DomainError;

static EMAIL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").expect("regex de email inválida"));

/// Value Object: um email só existe se for válido. Não há como construir
/// um `Email` inválido em nenhum ponto do sistema (Object Calisthenics:
/// "wrap all primitives").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let normalized = raw.trim().to_lowercase();
        if Self::is_invalid(&normalized) {
            return Err(DomainError::InvalidEmail);
        }
        Ok(Self(normalized))
    }

    fn is_invalid(candidate: &str) -> bool {
        !EMAIL_PATTERN.is_match(candidate)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

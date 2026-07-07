use once_cell::sync::Lazy;
use regex::Regex;

use crate::domain::shared::errors::DomainError;

static EMAIL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").expect("regex de email invalida"));

/// Value Object: um email so existe se for valido. Nao ha como construir
/// um `Email` invalido em nenhum ponto do sistema (Object Calisthenics:
/// "wrap all primitives"). Reaproveitado tanto por `User` quanto por
/// `Cliente` (Semana 10) - o conceito "email valido" e o mesmo nos dois
/// contextos, entao nao ha motivo para duplicar a validacao.
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

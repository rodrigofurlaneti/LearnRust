use crate::domain::errors::DomainError;

/// Value Object: senha em texto puro, so existe se atender a politica de
/// forca minima. Nunca e persistida nem logada - quem faz isso e
/// `HashedPassword`. (Nota de auditoria da Semana 10: por isso o log de
/// auditoria de `User` nunca inclui a senha, nem em texto puro nem em
/// hash - so campos nao sensiveis como email/role, ver
/// `application/audit_logger.rs`.)
#[derive(Clone)]
pub struct PlainPassword(String);

impl PlainPassword {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        if !Self::is_strong_enough(raw) {
            return Err(DomainError::WeakPassword);
        }
        Ok(Self(raw.to_string()))
    }

    fn is_strong_enough(raw: &str) -> bool {
        let has_min_length = raw.len() >= 8;
        let has_uppercase = raw.chars().any(|c| c.is_ascii_uppercase());
        let has_lowercase = raw.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = raw.chars().any(|c| c.is_ascii_digit());

        has_min_length && has_uppercase && has_lowercase && has_digit
    }

    /// So a camada de infraestrutura (hasher) tem motivo para ler o valor cru.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for PlainPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlainPassword(**redacted**)")
    }
}

/// Value Object: senha ja cifrada (bcrypt/argon2/etc). O dominio nao sabe
/// qual algoritmo foi usado - isso e decisao de infraestrutura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

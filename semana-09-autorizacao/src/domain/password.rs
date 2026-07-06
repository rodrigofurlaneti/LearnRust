use crate::domain::errors::DomainError;

/// Value Object: senha em texto puro, só existe se atender à política de
/// força mínima. Nunca é persistida nem logada — quem faz isso é
/// `HashedPassword`.
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

    /// Só a camada de infraestrutura (hasher) tem motivo para ler o valor cru.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for PlainPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlainPassword(**redacted**)")
    }
}

/// Value Object: senha já cifrada (bcrypt/argon2/etc). O domínio não sabe
/// qual algoritmo foi usado — isso é decisão de infraestrutura.
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

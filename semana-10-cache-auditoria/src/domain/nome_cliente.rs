use crate::domain::errors::DomainError;

/// Value Object: nome de cliente, so existe se tiver conteudo minimo
/// depois de aparado (`trim`). Mesma filosofia de `Email`/`PlainPassword`
/// - nenhuma `String` solta representa um nome dentro do sistema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NomeCliente(String);

const TAMANHO_MINIMO: usize = 2;

impl NomeCliente {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let normalized = raw.trim().to_string();
        if Self::is_too_short(&normalized) {
            return Err(DomainError::InvalidClienteName);
        }
        Ok(Self(normalized))
    }

    fn is_too_short(candidate: &str) -> bool {
        candidate.chars().count() < TAMANHO_MINIMO
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

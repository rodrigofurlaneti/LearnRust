use crate::domain::shared::errors::DomainError;

/// Value Object: nome generico, so existe se tiver conteudo minimo depois
/// de aparado (`trim`). Mesma filosofia de `Email`/`PlainPassword` -
/// nenhuma `String` solta representa um nome dentro do sistema.
///
/// Generalizado na Semana 11: na Semana 10 isto se chamava `NomeCliente`
/// e so era usado por `Cliente`. Como `Produto` tambem precisa de "um
/// nome que nao pode ser vazio", em vez de duplicar a mesma validacao
/// numa `NomeProduto` separada, promovemos o VO para `Nome` e os dois
/// agregados passam a compartilha-lo - o conceito "nome valido" e
/// identico nos dois contextos (mesmo raciocinio ja aplicado ao VO
/// `Email`, reaproveitado por `User` e `Cliente`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nome(String);

const TAMANHO_MINIMO: usize = 2;

impl Nome {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let normalized = raw.trim().to_string();
        if Self::is_too_short(&normalized) {
            return Err(DomainError::InvalidNome);
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

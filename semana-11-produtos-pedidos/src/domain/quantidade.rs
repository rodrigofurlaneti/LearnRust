use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

/// Value Object: quantidade de um item de pedido. So existe se for maior
/// que zero - "0 unidades de um produto" nao e um item de pedido, e a
/// ausencia dele.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantidade(u32);

impl Quantidade {
    pub fn parse(raw: u32) -> Result<Self, DomainError> {
        if raw == 0 {
            return Err(DomainError::InvalidQuantidade);
        }
        Ok(Self(raw))
    }

    pub fn valor(&self) -> u32 {
        self.0
    }
}

use uuid::Uuid;

/// Value Object de identidade do agregado `Pedido`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PedidoId(Uuid);

impl PedidoId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PedidoId {
    fn default() -> Self {
        Self::new()
    }
}

use uuid::Uuid;

/// Value Object de identidade do agregado `Produto`. Mesmo raciocinio de
/// `UserId`/`ClienteId`: nunca deixamos um `Uuid` cru circular pelo
/// sistema (Object Calisthenics: "wrap all primitives").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProdutoId(Uuid);

impl ProdutoId {
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

impl Default for ProdutoId {
    fn default() -> Self {
        Self::new()
    }
}

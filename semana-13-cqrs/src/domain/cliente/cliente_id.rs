use uuid::Uuid;

/// Value Object de identidade do agregado `Cliente`. Mesmo raciocinio de
/// `UserId`: nunca deixamos um `Uuid` cru circular pelo sistema (Object
/// Calisthenics: "wrap all primitives"). Sao dois VOs distintos - mesmo
/// que ambos "sejam" um `Uuid` por baixo - porque um `ClienteId` nunca
/// deveria poder ser passado onde um `UserId` e esperado (e vice-versa);
/// o compilador barra essa troca por engano.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClienteId(Uuid);

impl ClienteId {
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

impl Default for ClienteId {
    fn default() -> Self {
        Self::new()
    }
}

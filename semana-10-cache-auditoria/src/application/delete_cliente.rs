use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::domain::{ClienteId, ClienteRepository, DomainError};

/// Caso de uso de remocao de cliente (`DELETE /clientes/:id`).
pub struct DeleteCliente {
    repository: Arc<dyn ClienteRepository>,
}

impl DeleteCliente {
    pub fn new(repository: Arc<dyn ClienteRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, cliente_id: ClienteId) -> Result<(), ApplicationError> {
        self.ensure_cliente_exists(cliente_id).await?;
        self.repository.delete(cliente_id).await?;
        Ok(())
    }

    async fn ensure_cliente_exists(&self, cliente_id: ClienteId) -> Result<(), ApplicationError> {
        let existing = self.repository.find_by_id(cliente_id).await?;
        existing
            .map(|_| ())
            .ok_or(ApplicationError::Domain(DomainError::ClienteNotFound))
    }
}

use std::sync::Arc;

use crate::application::shared::dto::ClienteOutput;
use crate::application::shared::errors::ApplicationError;
use crate::application::cliente::register_cliente::RegisterCliente;
use crate::domain::{ClienteId, ClienteRepository, DomainError};

/// Caso de uso de consulta de um cliente pelo id (`GET /clientes/:id`).
pub struct GetCliente {
    repository: Arc<dyn ClienteRepository>,
}

impl GetCliente {
    pub fn new(repository: Arc<dyn ClienteRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, cliente_id: ClienteId) -> Result<ClienteOutput, ApplicationError> {
        let cliente = self
            .repository
            .find_by_id(cliente_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::ClienteNotFound))?;

        Ok(RegisterCliente::to_output(&cliente))
    }
}

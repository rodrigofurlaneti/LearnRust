use std::sync::Arc;

use crate::application::dto::ListClientesOutput;
use crate::application::errors::ApplicationError;
use crate::application::register_cliente::RegisterCliente;
use crate::domain::ClienteRepository;

/// Caso de uso de listagem de clientes (`GET /clientes`).
pub struct ListClientes {
    repository: Arc<dyn ClienteRepository>,
}

impl ListClientes {
    pub fn new(repository: Arc<dyn ClienteRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<ListClientesOutput, ApplicationError> {
        let clientes = self.repository.list_all().await?;
        let output = clientes.iter().map(RegisterCliente::to_output).collect();

        Ok(ListClientesOutput { clientes: output })
    }
}

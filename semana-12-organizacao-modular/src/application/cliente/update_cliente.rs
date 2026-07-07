use std::sync::Arc;

use crate::application::shared::dto::{ClienteOutput, UpdateClienteInput};
use crate::application::shared::errors::ApplicationError;
use crate::application::cliente::register_cliente::RegisterCliente;
use crate::domain::{Cliente, ClienteId, ClienteRepository, Cpf, DomainError, Email, Nome};

/// Caso de uso de atualizacao cadastral de cliente (`PUT /clientes/:id`).
pub struct UpdateCliente {
    repository: Arc<dyn ClienteRepository>,
}

impl UpdateCliente {
    pub fn new(repository: Arc<dyn ClienteRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        cliente_id: ClienteId,
        input: UpdateClienteInput,
    ) -> Result<ClienteOutput, ApplicationError> {
        let nome = Nome::parse(&input.nome)?;
        let documento = Cpf::parse(&input.documento)?;
        let email = Email::parse(&input.email)?;

        let existing_cliente = self.find_existing_cliente(cliente_id).await?;
        self.ensure_documento_is_available_for(&documento, cliente_id).await?;

        let updated_cliente = existing_cliente.with_updated_data(nome, documento, email);
        self.repository.update(&updated_cliente).await?;

        Ok(RegisterCliente::to_output(&updated_cliente))
    }

    async fn find_existing_cliente(&self, cliente_id: ClienteId) -> Result<Cliente, ApplicationError> {
        self.repository
            .find_by_id(cliente_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::ClienteNotFound))
    }

    async fn ensure_documento_is_available_for(
        &self,
        documento: &Cpf,
        cliente_id: ClienteId,
    ) -> Result<(), ApplicationError> {
        let existing = self.repository.find_by_documento(documento).await?;
        let belongs_to_someone_else = existing.map(|c| c.id() != cliente_id).unwrap_or(false);
        if belongs_to_someone_else {
            return Err(ApplicationError::Domain(DomainError::ClienteAlreadyExists));
        }
        Ok(())
    }
}

use std::sync::Arc;

use crate::application::dto::{ClienteOutput, RegisterClienteInput};
use crate::application::errors::ApplicationError;
use crate::domain::{Cliente, ClienteRepository, Cpf, DomainError, Email, NomeCliente};

/// Caso de uso de cadastro de cliente (novo na Semana 10), simetrico ao
/// `RegisterUser`: uma unica responsabilidade (SRP), depende so da
/// abstracao `ClienteRepository` (DIP). Nao sabe que existe cache nem
/// arquivo de auditoria por tras dela.
pub struct RegisterCliente {
    repository: Arc<dyn ClienteRepository>,
}

impl RegisterCliente {
    pub fn new(repository: Arc<dyn ClienteRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: RegisterClienteInput,
    ) -> Result<ClienteOutput, ApplicationError> {
        let nome = NomeCliente::parse(&input.nome)?;
        let documento = Cpf::parse(&input.documento)?;
        let email = Email::parse(&input.email)?;

        self.ensure_documento_is_available(&documento).await?;
        let cliente = Cliente::register(nome, documento, email);
        self.repository.save(&cliente).await?;

        Ok(Self::to_output(&cliente))
    }

    async fn ensure_documento_is_available(&self, documento: &Cpf) -> Result<(), ApplicationError> {
        let existing = self.repository.find_by_documento(documento).await?;
        if existing.is_some() {
            return Err(ApplicationError::Domain(DomainError::ClienteAlreadyExists));
        }
        Ok(())
    }

    pub(crate) fn to_output(cliente: &Cliente) -> ClienteOutput {
        ClienteOutput {
            cliente_id: cliente.id().as_uuid().to_string(),
            nome: cliente.nome().as_str().to_string(),
            documento: cliente.documento().as_str().to_string(),
            email: cliente.email().as_str().to_string(),
        }
    }
}

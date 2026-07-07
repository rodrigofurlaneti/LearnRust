//! Commands do agregado Cliente.

use async_trait::async_trait;

use crate::application::cliente::delete_cliente::DeleteCliente;
use crate::application::cliente::register_cliente::RegisterCliente;
use crate::application::cliente::update_cliente::UpdateCliente;
use crate::application::shared::cqrs::{Command, CommandHandler};
use crate::application::shared::dto::{ClienteOutput, RegisterClienteInput, UpdateClienteInput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::ClienteId;

/// Intencao de cadastrar um novo cliente.
pub struct RegisterClienteCommand(pub RegisterClienteInput);

impl Command for RegisterClienteCommand {
    type Output = ClienteOutput;
}

#[async_trait]
impl CommandHandler<RegisterClienteCommand> for RegisterCliente {
    async fn handle(&self, command: RegisterClienteCommand) -> Result<ClienteOutput, ApplicationError> {
        self.execute(command.0).await
    }
}

/// Intencao de atualizar o cadastro de um cliente existente.
pub struct UpdateClienteCommand {
    pub cliente_id: ClienteId,
    pub input: UpdateClienteInput,
}

impl Command for UpdateClienteCommand {
    type Output = ClienteOutput;
}

#[async_trait]
impl CommandHandler<UpdateClienteCommand> for UpdateCliente {
    async fn handle(&self, command: UpdateClienteCommand) -> Result<ClienteOutput, ApplicationError> {
        self.execute(command.cliente_id, command.input).await
    }
}

/// Intencao de remover um cliente.
pub struct DeleteClienteCommand {
    pub cliente_id: ClienteId,
}

impl Command for DeleteClienteCommand {
    type Output = ();
}

#[async_trait]
impl CommandHandler<DeleteClienteCommand> for DeleteCliente {
    async fn handle(&self, command: DeleteClienteCommand) -> Result<(), ApplicationError> {
        self.execute(command.cliente_id).await
    }
}

//! Commands do agregado Pedido. Sem `UpdatePedidoCommand` de proposito -
//! ver `domain::pedido::pedido_repository` para o racional (editar linhas
//! de um pedido ja feito e cancelar e refazer).

use async_trait::async_trait;

use crate::application::pedido::delete_pedido::DeletePedido;
use crate::application::pedido::register_pedido::RegisterPedido;
use crate::application::shared::cqrs::{Command, CommandHandler};
use crate::application::shared::dto::{PedidoOutput, RegisterPedidoInput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::PedidoId;

/// Intencao de criar um pedido (um ou mais itens, com preco/nome do
/// produto congelados no momento da criacao).
pub struct RegisterPedidoCommand(pub RegisterPedidoInput);

impl Command for RegisterPedidoCommand {
    type Output = PedidoOutput;
}

#[async_trait]
impl CommandHandler<RegisterPedidoCommand> for RegisterPedido {
    async fn handle(&self, command: RegisterPedidoCommand) -> Result<PedidoOutput, ApplicationError> {
        self.execute(command.0).await
    }
}

/// Intencao de cancelar (remover) um pedido.
pub struct DeletePedidoCommand {
    pub pedido_id: PedidoId,
}

impl Command for DeletePedidoCommand {
    type Output = ();
}

#[async_trait]
impl CommandHandler<DeletePedidoCommand> for DeletePedido {
    async fn handle(&self, command: DeletePedidoCommand) -> Result<(), ApplicationError> {
        self.execute(command.pedido_id).await
    }
}

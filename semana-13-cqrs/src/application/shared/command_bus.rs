//! `CommandBus`: unico ponto de entrada da presentation para qualquer
//! operacao de escrita. Por baixo ainda e so uma colecao de `Arc<UseCase>`
//! (nada muda em termos de dependencias reais), mas quem chama nao
//! precisa mais conhecer 12 tipos de caso de uso - so `Command`s e um
//! metodo generico `dispatch`.
//!
//! Nota de implementacao: em linguagens com reflection (C#, Java) um
//! Mediator tipicamente resolve o handler certo em tempo de execucao via
//! um registro dinamico (`Dictionary<Type, Handler>`). Em Rust isso exige
//! `TypeId` + downcasting (`dyn Any`), o que perde checagem em tempo de
//! compilacao e complica a assinatura de `handle` (que e generica sobre
//! `C::Output`). Por isso o `CommandBus` aqui e resolvido em tempo de
//! compilacao: cada Command tem um `impl CommandHandler<Command> for
//! CommandBus` proprio, e `dispatch::<C>` so pede ao compilador para
//! escolher o `impl` certo - mesmo resultado (presentation so ve
//! `Command`s), sem abrir mao da seguranca de tipos.

use std::sync::Arc;

use async_trait::async_trait;

use crate::application::cliente::commands::{
    DeleteClienteCommand, RegisterClienteCommand, UpdateClienteCommand,
};
use crate::application::cliente::delete_cliente::DeleteCliente;
use crate::application::cliente::register_cliente::RegisterCliente;
use crate::application::cliente::update_cliente::UpdateCliente;
use crate::application::pedido::commands::{DeletePedidoCommand, RegisterPedidoCommand};
use crate::application::pedido::delete_pedido::DeletePedido;
use crate::application::pedido::register_pedido::RegisterPedido;
use crate::application::produto::commands::{
    DeleteProdutoCommand, RegisterProdutoCommand, UpdateProdutoCommand,
};
use crate::application::produto::delete_produto::DeleteProduto;
use crate::application::produto::register_produto::RegisterProduto;
use crate::application::produto::update_produto::UpdateProduto;
use crate::application::shared::cqrs::{Command, CommandHandler};
use crate::application::shared::dto::{
    AuthenticateUserOutput, ClienteOutput, PedidoOutput, ProdutoOutput, RegisterUserOutput,
    UpdateUserOutput,
};
use crate::application::shared::errors::ApplicationError;
use crate::application::usuario::authenticate_user::AuthenticateUser;
use crate::application::usuario::commands::{
    AuthenticateUserCommand, DeleteUserCommand, RegisterUserCommand, UpdateUserCommand,
};
use crate::application::usuario::delete_user::DeleteUser;
use crate::application::usuario::register_user::RegisterUser;
use crate::application::usuario::update_user::UpdateUser;

pub struct CommandBus {
    register_user: Arc<RegisterUser>,
    authenticate_user: Arc<AuthenticateUser>,
    update_user: Arc<UpdateUser>,
    delete_user: Arc<DeleteUser>,
    register_cliente: Arc<RegisterCliente>,
    update_cliente: Arc<UpdateCliente>,
    delete_cliente: Arc<DeleteCliente>,
    register_produto: Arc<RegisterProduto>,
    update_produto: Arc<UpdateProduto>,
    delete_produto: Arc<DeleteProduto>,
    register_pedido: Arc<RegisterPedido>,
    delete_pedido: Arc<DeletePedido>,
}

impl CommandBus {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        register_user: Arc<RegisterUser>,
        authenticate_user: Arc<AuthenticateUser>,
        update_user: Arc<UpdateUser>,
        delete_user: Arc<DeleteUser>,
        register_cliente: Arc<RegisterCliente>,
        update_cliente: Arc<UpdateCliente>,
        delete_cliente: Arc<DeleteCliente>,
        register_produto: Arc<RegisterProduto>,
        update_produto: Arc<UpdateProduto>,
        delete_produto: Arc<DeleteProduto>,
        register_pedido: Arc<RegisterPedido>,
        delete_pedido: Arc<DeletePedido>,
    ) -> Self {
        Self {
            register_user,
            authenticate_user,
            update_user,
            delete_user,
            register_cliente,
            update_cliente,
            delete_cliente,
            register_produto,
            update_produto,
            delete_produto,
            register_pedido,
            delete_pedido,
        }
    }

    /// Porta de entrada unica usada pela presentation: `bus.dispatch(SomeCommand{...}).await`.
    /// O compilador escolhe o `impl CommandHandler<C> for CommandBus` certo
    /// a partir do tipo concreto de `C` - nao ha nenhuma resolucao em tempo
    /// de execucao.
    pub async fn dispatch<C>(&self, command: C) -> Result<C::Output, ApplicationError>
    where
        C: Command,
        Self: CommandHandler<C>,
    {
        CommandHandler::handle(self, command).await
    }
}

#[async_trait]
impl CommandHandler<RegisterUserCommand> for CommandBus {
    async fn handle(&self, command: RegisterUserCommand) -> Result<RegisterUserOutput, ApplicationError> {
        self.register_user.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<AuthenticateUserCommand> for CommandBus {
    async fn handle(
        &self,
        command: AuthenticateUserCommand,
    ) -> Result<AuthenticateUserOutput, ApplicationError> {
        self.authenticate_user.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<UpdateUserCommand> for CommandBus {
    async fn handle(&self, command: UpdateUserCommand) -> Result<UpdateUserOutput, ApplicationError> {
        self.update_user.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<DeleteUserCommand> for CommandBus {
    async fn handle(&self, command: DeleteUserCommand) -> Result<(), ApplicationError> {
        self.delete_user.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<RegisterClienteCommand> for CommandBus {
    async fn handle(&self, command: RegisterClienteCommand) -> Result<ClienteOutput, ApplicationError> {
        self.register_cliente.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<UpdateClienteCommand> for CommandBus {
    async fn handle(&self, command: UpdateClienteCommand) -> Result<ClienteOutput, ApplicationError> {
        self.update_cliente.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<DeleteClienteCommand> for CommandBus {
    async fn handle(&self, command: DeleteClienteCommand) -> Result<(), ApplicationError> {
        self.delete_cliente.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<RegisterProdutoCommand> for CommandBus {
    async fn handle(&self, command: RegisterProdutoCommand) -> Result<ProdutoOutput, ApplicationError> {
        self.register_produto.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<UpdateProdutoCommand> for CommandBus {
    async fn handle(&self, command: UpdateProdutoCommand) -> Result<ProdutoOutput, ApplicationError> {
        self.update_produto.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<DeleteProdutoCommand> for CommandBus {
    async fn handle(&self, command: DeleteProdutoCommand) -> Result<(), ApplicationError> {
        self.delete_produto.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<RegisterPedidoCommand> for CommandBus {
    async fn handle(&self, command: RegisterPedidoCommand) -> Result<PedidoOutput, ApplicationError> {
        self.register_pedido.handle(command).await
    }
}

#[async_trait]
impl CommandHandler<DeletePedidoCommand> for CommandBus {
    async fn handle(&self, command: DeletePedidoCommand) -> Result<(), ApplicationError> {
        self.delete_pedido.handle(command).await
    }
}

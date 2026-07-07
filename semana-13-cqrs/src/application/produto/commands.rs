//! Commands do agregado Produto.

use async_trait::async_trait;

use crate::application::produto::delete_produto::DeleteProduto;
use crate::application::produto::register_produto::RegisterProduto;
use crate::application::produto::update_produto::UpdateProduto;
use crate::application::shared::cqrs::{Command, CommandHandler};
use crate::application::shared::dto::{ProdutoOutput, RegisterProdutoInput, UpdateProdutoInput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::ProdutoId;

/// Intencao de cadastrar um novo produto.
pub struct RegisterProdutoCommand(pub RegisterProdutoInput);

impl Command for RegisterProdutoCommand {
    type Output = ProdutoOutput;
}

#[async_trait]
impl CommandHandler<RegisterProdutoCommand> for RegisterProduto {
    async fn handle(&self, command: RegisterProdutoCommand) -> Result<ProdutoOutput, ApplicationError> {
        self.execute(command.0).await
    }
}

/// Intencao de atualizar o cadastro de um produto existente.
pub struct UpdateProdutoCommand {
    pub produto_id: ProdutoId,
    pub input: UpdateProdutoInput,
}

impl Command for UpdateProdutoCommand {
    type Output = ProdutoOutput;
}

#[async_trait]
impl CommandHandler<UpdateProdutoCommand> for UpdateProduto {
    async fn handle(&self, command: UpdateProdutoCommand) -> Result<ProdutoOutput, ApplicationError> {
        self.execute(command.produto_id, command.input).await
    }
}

/// Intencao de remover um produto.
pub struct DeleteProdutoCommand {
    pub produto_id: ProdutoId,
}

impl Command for DeleteProdutoCommand {
    type Output = ();
}

#[async_trait]
impl CommandHandler<DeleteProdutoCommand> for DeleteProduto {
    async fn handle(&self, command: DeleteProdutoCommand) -> Result<(), ApplicationError> {
        self.execute(command.produto_id).await
    }
}

//! Commands do agregado Usuario: registro publico, login, e gestao
//! administrativa (update/delete). Cada struct e um adaptador fino sobre
//! o caso de uso que ja existia - nenhuma regra de negocio muda aqui.

use async_trait::async_trait;

use crate::application::shared::cqrs::{Command, CommandHandler};
use crate::application::shared::dto::{
    AuthenticateUserInput, AuthenticateUserOutput, RegisterUserInput, RegisterUserOutput,
    UpdateUserInput, UpdateUserOutput,
};
use crate::application::shared::errors::ApplicationError;
use crate::application::usuario::authenticate_user::AuthenticateUser;
use crate::application::usuario::delete_user::DeleteUser;
use crate::application::usuario::register_user::RegisterUser;
use crate::application::usuario::update_user::UpdateUser;
use crate::domain::UserId;

/// Intencao de cadastrar um novo usuario.
pub struct RegisterUserCommand(pub RegisterUserInput);

impl Command for RegisterUserCommand {
    type Output = RegisterUserOutput;
}

#[async_trait]
impl CommandHandler<RegisterUserCommand> for RegisterUser {
    async fn handle(&self, command: RegisterUserCommand) -> Result<RegisterUserOutput, ApplicationError> {
        self.execute(command.0).await
    }
}

/// Intencao de autenticar um usuario e emitir um access token. Nao muda o
/// estado do agregado `User`, mas e tratada como Command (e nao Query)
/// porque e uma acao imperativa - "faca login" - com efeitos que vao alem
/// de so ler dados (emissao de token, rate limiting).
pub struct AuthenticateUserCommand(pub AuthenticateUserInput);

impl Command for AuthenticateUserCommand {
    type Output = AuthenticateUserOutput;
}

#[async_trait]
impl CommandHandler<AuthenticateUserCommand> for AuthenticateUser {
    async fn handle(
        &self,
        command: AuthenticateUserCommand,
    ) -> Result<AuthenticateUserOutput, ApplicationError> {
        self.execute(command.0).await
    }
}

/// Intencao de atualizar o cadastro de um usuario existente (somente
/// admin - a regra de autorizacao continua sendo responsabilidade da
/// presentation, o Command so carrega a intencao).
pub struct UpdateUserCommand {
    pub user_id: UserId,
    pub input: UpdateUserInput,
}

impl Command for UpdateUserCommand {
    type Output = UpdateUserOutput;
}

#[async_trait]
impl CommandHandler<UpdateUserCommand> for UpdateUser {
    async fn handle(&self, command: UpdateUserCommand) -> Result<UpdateUserOutput, ApplicationError> {
        self.execute(command.user_id, command.input).await
    }
}

/// Intencao de remover um usuario.
pub struct DeleteUserCommand {
    pub user_id: UserId,
}

impl Command for DeleteUserCommand {
    type Output = ();
}

#[async_trait]
impl CommandHandler<DeleteUserCommand> for DeleteUser {
    async fn handle(&self, command: DeleteUserCommand) -> Result<(), ApplicationError> {
        self.execute(command.user_id).await
    }
}

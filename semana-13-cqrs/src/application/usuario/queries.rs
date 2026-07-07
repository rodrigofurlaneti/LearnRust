//! Queries do agregado Usuario.

use async_trait::async_trait;

use crate::application::shared::cqrs::{Query, QueryHandler};
use crate::application::shared::dto::CurrentUserOutput;
use crate::application::shared::errors::ApplicationError;
use crate::application::usuario::get_current_user::GetCurrentUser;
use crate::domain::UserId;

/// Pergunta pelos dados do usuario atualmente autenticado.
pub struct GetCurrentUserQuery {
    pub user_id: UserId,
}

impl Query for GetCurrentUserQuery {
    type Output = CurrentUserOutput;
}

#[async_trait]
impl QueryHandler<GetCurrentUserQuery> for GetCurrentUser {
    async fn handle(&self, query: GetCurrentUserQuery) -> Result<CurrentUserOutput, ApplicationError> {
        self.execute(query.user_id).await
    }
}

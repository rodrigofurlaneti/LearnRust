use std::sync::Arc;

use crate::application::authenticate_user::AuthenticateUser;
use crate::application::get_current_user::GetCurrentUser;
use crate::application::register_user::RegisterUser;
use crate::application::token_service::TokenService;

/// Estado compartilhado do Axum. E aqui, e so aqui, que a presentation
/// conhece os casos de uso concretos - ela nunca conhece repositorio,
/// hasher ou implementacao de JWT diretamente (isso e resolvido no
/// `main.rs`). `tokens` fica exposto tambem aqui (alem de dentro do
/// `AuthenticateUser`) porque o extractor `AuthenticatedUser` precisa
/// verificar tokens em toda rota protegida, nao so no login.
#[derive(Clone)]
pub struct AppState {
    pub register_user: Arc<RegisterUser>,
    pub authenticate_user: Arc<AuthenticateUser>,
    pub get_current_user: Arc<GetCurrentUser>,
    pub tokens: Arc<dyn TokenService>,
}

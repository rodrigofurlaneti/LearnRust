use std::sync::Arc;

use crate::application::authenticate_user::AuthenticateUser;
use crate::application::register_user::RegisterUser;

/// Estado compartilhado do Axum. É aqui, e só aqui, que a presentation
/// conhece os casos de uso concretos — ela nunca conhece repositório,
/// hasher ou JWT diretamente (isso é resolvido no `main.rs`).
#[derive(Clone)]
pub struct AppState {
    pub register_user: Arc<RegisterUser>,
    pub authenticate_user: Arc<AuthenticateUser>,
}

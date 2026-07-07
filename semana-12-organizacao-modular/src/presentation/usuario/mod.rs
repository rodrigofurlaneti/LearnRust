//! Handlers HTTP do agregado Usuario (autenticacao/registro publico e
//! gestao administrativa).

pub mod auth_handlers;
pub mod user_handlers;

pub use auth_handlers::{admin_ping, login, me, register};
pub use user_handlers::{delete_user, update_user};

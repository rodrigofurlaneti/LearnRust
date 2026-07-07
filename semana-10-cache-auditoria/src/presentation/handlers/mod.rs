pub mod auth_handlers;
pub mod cliente_handlers;
pub mod user_handlers;

pub use auth_handlers::{admin_ping, login, me, register};
pub use cliente_handlers::{create_cliente, delete_cliente, get_cliente, list_clientes, update_cliente};
pub use user_handlers::{delete_user, update_user};

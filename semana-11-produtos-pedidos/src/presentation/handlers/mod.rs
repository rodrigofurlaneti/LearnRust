pub mod auth_handlers;
pub mod cliente_handlers;
pub mod pedido_handlers;
pub mod produto_handlers;
pub mod user_handlers;

pub use auth_handlers::{admin_ping, login, me, register};
pub use cliente_handlers::{create_cliente, delete_cliente, get_cliente, list_clientes, update_cliente};
pub use pedido_handlers::{create_pedido, delete_pedido, get_pedido, list_pedidos};
pub use produto_handlers::{create_produto, delete_produto, get_produto, list_produtos, update_produto};
pub use user_handlers::{delete_user, update_user};

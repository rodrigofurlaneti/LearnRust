//! Camada de apresentacao (Axum), organizada por agregado (Semana 12): os
//! handlers HTTP de cada agregado moram no submodulo correspondente;
//! `shared` guarda o que atravessa todos eles (extractors, rotas, estado,
//! OpenAPI). Os handlers continuam reexportados aqui no nivel do modulo
//! `presentation`, entao `crate::presentation::create_cliente` etc.
//! seguem acessiveis sem que `routes.rs` precise saber em qual agregado
//! cada handler mora.

pub mod cliente;
pub mod pedido;
pub mod produto;
pub mod shared;
pub mod usuario;

pub use cliente::{create_cliente, delete_cliente, get_cliente, list_clientes, update_cliente};
pub use pedido::{create_pedido, delete_pedido, get_pedido, list_pedidos};
pub use produto::{create_produto, delete_produto, get_produto, list_produtos, update_produto};
pub use usuario::{admin_ping, delete_user, login, me, register, update_user};

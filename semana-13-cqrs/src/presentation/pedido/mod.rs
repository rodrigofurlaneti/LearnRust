//! Handlers HTTP do agregado Pedido (sem PUT de proposito - ver
//! `domain::pedido::pedido_repository`).

pub mod pedido_handlers;

pub use pedido_handlers::{create_pedido, delete_pedido, get_pedido, list_pedidos};

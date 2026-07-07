//! Casos de uso do agregado Pedido. Nao ha `update_pedido` de proposito
//! (ver `domain::pedido::pedido_repository` para o racional).
//! `commands.rs`/`queries.rs` (Semana 13) formalizam CQRS em cima destes
//! casos de uso, sem alterar `execute`.

pub mod commands;
pub mod delete_pedido;
pub mod get_pedido;
pub mod list_pedidos;
pub mod queries;
pub mod register_pedido;

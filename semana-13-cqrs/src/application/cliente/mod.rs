//! Casos de uso do agregado Cliente. `commands.rs`/`queries.rs` (Semana
//! 13) formalizam CQRS em cima destes casos de uso, sem alterar `execute`.

pub mod commands;
pub mod delete_cliente;
pub mod get_cliente;
pub mod list_clientes;
pub mod queries;
pub mod register_cliente;
pub mod update_cliente;

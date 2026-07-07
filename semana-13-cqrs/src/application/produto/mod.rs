//! Casos de uso do agregado Produto. `commands.rs`/`queries.rs` (Semana
//! 13) formalizam CQRS em cima destes casos de uso, sem alterar `execute`.

pub mod commands;
pub mod delete_produto;
pub mod get_produto;
pub mod list_produtos;
pub mod queries;
pub mod register_produto;
pub mod update_produto;

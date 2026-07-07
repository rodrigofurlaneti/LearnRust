//! Casos de uso do agregado Usuario. `commands.rs`/`queries.rs` (Semana
//! 13) formalizam CQRS em cima destes casos de uso, sem alterar `execute`.

pub mod authenticate_user;
pub mod commands;
pub mod delete_user;
pub mod get_current_user;
pub mod queries;
pub mod register_user;
pub mod update_user;

//! Portas e tipos de aplicacao usados por mais de um agregado: o logger de
//! auditoria, o vocabulario de CQRS (Command/Query/Handler/Bus), os DTOs
//! de entrada/saida, os erros de aplicacao e o servico de tokens.

pub mod audit_logger;
pub mod command_bus;
pub mod cqrs;
pub mod dto;
pub mod errors;
pub mod query_bus;
pub mod token_service;

pub use command_bus::CommandBus;
pub use cqrs::{Command, CommandHandler, Query, QueryHandler};
pub use query_bus::QueryBus;

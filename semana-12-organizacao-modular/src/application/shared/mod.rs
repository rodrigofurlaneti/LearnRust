//! Portas e tipos de aplicacao usados por mais de um agregado: o logger de
//! auditoria, os DTOs de entrada/saida, os erros de aplicacao e o servico
//! de tokens.

pub mod audit_logger;
pub mod dto;
pub mod errors;
pub mod token_service;

//! Implementacoes concretas do agregado Usuario: cache em memoria, o
//! decorator de auditoria, o hasher de senha (bcrypt) e o servico de
//! tokens (JWT).

pub mod audited_user_repository;
pub mod bcrypt_hasher;
pub mod cache_user_repository;
pub mod jwt_token_service;

//! Agregado Cliente.

pub mod entidade;
pub mod cliente_id;
pub mod cliente_repository;
pub mod cpf;

pub use entidade::Cliente;
pub use cliente_id::ClienteId;
pub use cliente_repository::ClienteRepository;
pub use cpf::Cpf;

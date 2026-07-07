//! Value Objects e tipos de erro usados por mais de um agregado (Dinheiro,
//! Email, Nome e Quantidade nao pertencem a nenhum agregado especifico).

pub mod dinheiro;
pub mod email;
pub mod errors;
pub mod nome;
pub mod quantidade;

pub use dinheiro::Dinheiro;
pub use email::Email;
pub use errors::DomainError;
pub use nome::Nome;
pub use quantidade::Quantidade;

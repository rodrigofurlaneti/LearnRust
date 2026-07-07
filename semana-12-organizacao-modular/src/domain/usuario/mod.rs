//! Agregado Usuario: autenticacao, autorizacao (Role) e o contrato de
//! persistencia (UserRepository). O que e compartilhado com outros
//! agregados (Email, por exemplo) fica em `domain::shared`.

pub mod password;
pub mod password_hasher;
pub mod role;
pub mod user;
pub mod user_id;
pub mod user_repository;

pub use password::{HashedPassword, PlainPassword};
pub use password_hasher::PasswordHasher;
pub use role::Role;
pub use user::User;
pub use user_id::UserId;
pub use user_repository::UserRepository;

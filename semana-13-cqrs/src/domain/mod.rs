//! Camada de dominio, organizada por agregado (Semana 12). Cada agregado
//! (usuario, cliente, produto, pedido) e um submodulo com suas proprias
//! entidades, Value Objects e o contrato de repositorio; `shared` guarda o
//! que atravessa mais de um agregado. Os tipos publicos continuam
//! reexportados aqui no nivel do modulo `domain`, entao
//! `crate::domain::User`, `crate::domain::ClienteRepository` etc. seguem
//! funcionando sem mudanca para quem consome esta camada de fora.

pub mod cliente;
pub mod pedido;
pub mod produto;
pub mod shared;
pub mod usuario;

pub use cliente::{Cliente, ClienteId, ClienteRepository, Cpf};
pub use pedido::{ItemPedido, Pedido, PedidoId, PedidoRepository};
pub use produto::{Produto, ProdutoId, ProdutoRepository};
pub use shared::{Dinheiro, DomainError, Email, Nome, Quantidade};
pub use usuario::{HashedPassword, PasswordHasher, PlainPassword, Role, User, UserId, UserRepository};

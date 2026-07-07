//! Agregado Pedido, incluindo o Value Object ItemPedido (que congela nome e
//! preco do produto no momento da compra).

pub mod item_pedido;
pub mod entidade;
pub mod pedido_id;
pub mod pedido_repository;

pub use item_pedido::ItemPedido;
pub use entidade::Pedido;
pub use pedido_id::PedidoId;
pub use pedido_repository::PedidoRepository;

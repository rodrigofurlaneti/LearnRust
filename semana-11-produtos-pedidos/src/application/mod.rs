pub mod audit_logger;
pub mod authenticate_user;
pub mod delete_cliente;
pub mod delete_pedido;
pub mod delete_produto;
pub mod delete_user;
pub mod dto;
pub mod errors;
pub mod get_cliente;
pub mod get_current_user;
pub mod get_pedido;
pub mod get_produto;
pub mod list_clientes;
pub mod list_pedidos;
pub mod list_produtos;
pub mod register_cliente;
pub mod register_pedido;
pub mod register_produto;
pub mod register_user;
pub mod token_service;
pub mod update_cliente;
pub mod update_produto;
pub mod update_user;

// Reexportados para quem for consumir esta camada de fora (ex.: presentation
// ou testes de integracao) sem precisar conhecer os modulos internos.
#[allow(unused_imports)]
pub use audit_logger::{AuditAction, AuditEntity, AuditEntry, AuditLogger, AuditPayload, ItemPedidoSnapshot};
#[allow(unused_imports)]
pub use errors::ApplicationError;
#[allow(unused_imports)]
pub use token_service::{AccessToken, TokenClaims, TokenService};

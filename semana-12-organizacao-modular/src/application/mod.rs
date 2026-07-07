//! Camada de aplicacao, organizada por agregado (Semana 12): cada caso de
//! uso mora no submodulo do agregado que ele orquestra; `shared` guarda o
//! que atravessa mais de um agregado (auditoria, DTOs, erros, tokens).

pub mod cliente;
pub mod pedido;
pub mod produto;
pub mod shared;
pub mod usuario;

// Reexportados para quem for consumir esta camada de fora (ex.: presentation
// ou testes de integracao) sem precisar conhecer os modulos internos.
#[allow(unused_imports)]
pub use shared::audit_logger::{
    AuditAction, AuditEntity, AuditEntry, AuditLogger, AuditPayload, ItemPedidoSnapshot,
};
#[allow(unused_imports)]
pub use shared::errors::ApplicationError;
#[allow(unused_imports)]
pub use shared::token_service::{AccessToken, TokenClaims, TokenService};

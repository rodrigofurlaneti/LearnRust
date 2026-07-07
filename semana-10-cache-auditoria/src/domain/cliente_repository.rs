use async_trait::async_trait;

use crate::domain::cliente::Cliente;
use crate::domain::cliente_id::ClienteId;
use crate::domain::cpf::Cpf;
use crate::domain::errors::DomainError;

/// Porta de saida (Dependency Inversion Principle) do agregado `Cliente`,
/// espelhando `UserRepository`: dominio e aplicacao dependem so desta
/// abstracao. Quem implementa, na Semana 10, e o cache em memoria
/// (`infra::cache_cliente_repository`) decorado com auditoria
/// (`infra::audited_cliente_repository`).
#[async_trait]
pub trait ClienteRepository: Send + Sync {
    async fn save(&self, cliente: &Cliente) -> Result<(), DomainError>;
    async fn update(&self, cliente: &Cliente) -> Result<(), DomainError>;
    async fn delete(&self, id: ClienteId) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: ClienteId) -> Result<Option<Cliente>, DomainError>;
    async fn find_by_documento(&self, documento: &Cpf) -> Result<Option<Cliente>, DomainError>;
    async fn list_all(&self) -> Result<Vec<Cliente>, DomainError>;
}

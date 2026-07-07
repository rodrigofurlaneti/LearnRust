use async_trait::async_trait;

use crate::domain::errors::DomainError;
use crate::domain::pedido::Pedido;
use crate::domain::pedido_id::PedidoId;

/// Porta de saida do agregado `Pedido`. De proposito NAO tem `update`:
/// editar as linhas de um pedido ja feito (trocar produto/quantidade,
/// recalcular totais) e uma operacao de negocio bem mais delicada que
/// "trocar um email" - decidimos deixar fora do escopo desta semana (ver
/// README, secao "proximos passos") em vez de modelar isso pela metade.
/// Um pedido incorreto e cancelado (`delete`) e refeito, nao editado.
#[async_trait]
pub trait PedidoRepository: Send + Sync {
    async fn save(&self, pedido: &Pedido) -> Result<(), DomainError>;
    async fn delete(&self, id: PedidoId) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: PedidoId) -> Result<Option<Pedido>, DomainError>;
    async fn list_all(&self) -> Result<Vec<Pedido>, DomainError>;
}

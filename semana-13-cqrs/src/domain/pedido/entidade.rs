use crate::domain::cliente::cliente_id::ClienteId;
use crate::domain::shared::dinheiro::Dinheiro;
use crate::domain::shared::errors::DomainError;
use crate::domain::pedido::item_pedido::ItemPedido;
use crate::domain::pedido::pedido_id::PedidoId;

/// Entidade raiz de agregado (novo na Semana 11): um pedido pertence a um
/// `Cliente` e tem uma ou mais linhas (`ItemPedido`). `Pedido` e o
/// AGGREGATE ROOT - `ItemPedido` nunca e acessado/persistido fora de um
/// `Pedido` (mesmo padrao Aggregate Root + Value Object interno visto na
/// Semana 2, `dominio-pedidos`).
///
/// Invariante de negocio garantida no construtor: um pedido sem nenhum
/// item nao e um pedido valido - `register` recusa a lista vazia
/// (`DomainError::PedidoSemItens`), estruturalmente impossivel de burlar
/// (nao existe nenhum outro jeito publico de criar um `Pedido`).
///
/// Deriva `Clone` pela mesma razao de `User`/`Cliente`/`Produto`: o cache
/// em memoria (`infra::cache_pedido_repository`) guarda o agregado inteiro
/// num `HashMap` atras de um `RwLock` e devolve copias em cada leitura,
/// nunca uma referencia viva presa dentro do lock.
#[derive(Clone)]
pub struct Pedido {
    id: PedidoId,
    cliente_id: ClienteId,
    itens: Vec<ItemPedido>,
    valor_total: Dinheiro,
}

impl Pedido {
    /// Fluxo publico de criacao (`POST /pedidos`, via caso de uso
    /// `RegisterPedido`). Quem monta cada `ItemPedido` com o preco
    /// congelado e o caso de uso (ele e quem fala com o
    /// `ProdutoRepository`) - o dominio so garante o invariante "pelo
    /// menos um item" e soma os totais.
    pub fn register(cliente_id: ClienteId, itens: Vec<ItemPedido>) -> Result<Self, DomainError> {
        if itens.is_empty() {
            return Err(DomainError::PedidoSemItens);
        }

        let valor_total = itens.iter().map(ItemPedido::valor_total).sum();

        Ok(Self {
            id: PedidoId::new(),
            cliente_id,
            itens,
            valor_total,
        })
    }

    /// Reidrata um pedido ja existente - usado pelo cache e pelo replay
    /// do log de auditoria no boot.
    pub fn reconstitute(
        id: PedidoId,
        cliente_id: ClienteId,
        itens: Vec<ItemPedido>,
        valor_total: Dinheiro,
    ) -> Self {
        Self {
            id,
            cliente_id,
            itens,
            valor_total,
        }
    }

    pub fn id(&self) -> PedidoId {
        self.id
    }

    pub fn cliente_id(&self) -> ClienteId {
        self.cliente_id
    }

    pub fn itens(&self) -> &[ItemPedido] {
        &self.itens
    }

    pub fn valor_total(&self) -> Dinheiro {
        self.valor_total
    }
}

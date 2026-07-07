use crate::domain::shared::dinheiro::Dinheiro;
use crate::domain::shared::nome::Nome;
use crate::domain::produto::produto_id::ProdutoId;

/// Entidade raiz de agregado (novo na Semana 11): representa o cadastro
/// de produto. So dois campos alem do id - `nome` e `preco` - ambos
/// Value Objects coesos.
///
/// Nao ha controle de estoque de proposito (fora do escopo pedido para
/// esta semana - ver README, secao "proximos passos"): remover um
/// produto do cadastro nunca invalida pedidos ja feitos, porque
/// `ItemPedido` guarda uma COPIA do nome/preco no momento da compra (ver
/// `domain/item_pedido.rs`) - o produto pode sumir do cadastro sem que o
/// historico de pedidos quebre.
#[derive(Clone)]
pub struct Produto {
    id: ProdutoId,
    nome: Nome,
    preco: Dinheiro,
}

impl Produto {
    pub fn register(nome: Nome, preco: Dinheiro) -> Self {
        Self {
            id: ProdutoId::new(),
            nome,
            preco,
        }
    }

    /// Reidrata um produto ja existente - usado pela camada de
    /// infraestrutura (cache) e pelo replay do log de auditoria no boot.
    pub fn reconstitute(id: ProdutoId, nome: Nome, preco: Dinheiro) -> Self {
        Self { id, nome, preco }
    }

    /// Regra de negocio de atualizacao cadastral: troca nome/preco
    /// mantendo a identidade. Devolve um novo `Produto` - o agregado
    /// continua imutavel de fora.
    pub fn with_updated_data(&self, nome: Nome, preco: Dinheiro) -> Self {
        Self {
            id: self.id,
            nome,
            preco,
        }
    }

    pub fn id(&self) -> ProdutoId {
        self.id
    }

    pub fn nome(&self) -> &Nome {
        &self.nome
    }

    pub fn preco(&self) -> Dinheiro {
        self.preco
    }
}

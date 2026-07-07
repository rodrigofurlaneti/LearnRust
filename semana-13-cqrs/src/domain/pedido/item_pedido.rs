use crate::domain::shared::dinheiro::Dinheiro;
use crate::domain::shared::nome::Nome;
use crate::domain::produto::produto_id::ProdutoId;
use crate::domain::shared::quantidade::Quantidade;

/// Value Object filho do agregado `Pedido` (nao tem repositorio proprio -
/// so existe dentro de um `Pedido`, e e sempre persistido/lido junto com
/// ele). Este e o coracao da regra de negocio pedida na Semana 11: guarda
/// uma COPIA do nome e do preco do produto no exato momento da compra,
/// nao uma referencia viva a `Produto`.
///
/// Por que isso importa: se `ItemPedido` guardasse so `produto_id` e
/// `quantidade`, calculando o valor "na hora de exibir" via
/// `ProdutoRepository`, um pedido de mes passado mudaria de valor total
/// sempre que o preco do produto mudasse hoje - o que e simplesmente
/// errado do ponto de vista de negocio (e o mesmo motivo pelo qual uma
/// nota fiscal real imprime o preco praticado na venda, nao o preco atual
/// da prateleira). Congelando `valor_unitario` (e `nome_produto`, para o
/// pedido continuar legivel mesmo que o produto seja renomeado ou
/// removido depois) no momento da criacao do pedido, o valor de um
/// pedido antigo nunca muda.
#[derive(Debug, Clone)]
pub struct ItemPedido {
    produto_id: ProdutoId,
    nome_produto: Nome,
    quantidade: Quantidade,
    valor_unitario: Dinheiro,
    valor_total: Dinheiro,
}

impl ItemPedido {
    /// Usado pelo caso de uso `RegisterPedido`: recebe o preco ATUAL do
    /// produto (lido do `ProdutoRepository` no momento do pedido) e
    /// congela o snapshot. O calculo de `valor_total` mora aqui dentro
    /// (Tell, Don't Ask) - quem chama nunca multiplica preco por
    /// quantidade na mao.
    pub fn snapshot_no_momento_do_pedido(
        produto_id: ProdutoId,
        nome_produto: Nome,
        quantidade: Quantidade,
        valor_unitario_atual: Dinheiro,
    ) -> Self {
        let valor_total = valor_unitario_atual.multiplicar_por(quantidade);
        Self {
            produto_id,
            nome_produto,
            quantidade,
            valor_unitario: valor_unitario_atual,
            valor_total,
        }
    }

    /// Reidrata um item ja persistido (cache) ou replayado do log de
    /// auditoria - os valores ja foram calculados/validados da primeira
    /// vez, entao so recompomos a struct.
    pub fn reconstitute(
        produto_id: ProdutoId,
        nome_produto: Nome,
        quantidade: Quantidade,
        valor_unitario: Dinheiro,
        valor_total: Dinheiro,
    ) -> Self {
        Self {
            produto_id,
            nome_produto,
            quantidade,
            valor_unitario,
            valor_total,
        }
    }

    pub fn produto_id(&self) -> ProdutoId {
        self.produto_id
    }

    pub fn nome_produto(&self) -> &Nome {
        &self.nome_produto
    }

    pub fn quantidade(&self) -> Quantidade {
        self.quantidade
    }

    pub fn valor_unitario(&self) -> Dinheiro {
        self.valor_unitario
    }

    pub fn valor_total(&self) -> Dinheiro {
        self.valor_total
    }
}

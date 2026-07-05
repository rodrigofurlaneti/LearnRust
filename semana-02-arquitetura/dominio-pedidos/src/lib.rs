pub struct ItemPedido {
    produto: String,
    quantidade: u32,
    preco_unitario: f64,
}

impl ItemPedido {
    pub fn new(produto: &str, quantidade: u32, preco_unitario: f64) -> Result<Self, String> {
        if quantidade == 0 {
            return Err("quantidade deve ser maior que zero".to_string());
        }
        if preco_unitario <= 0.0 {
            return Err("preco unitario deve ser maior que zero".to_string());
        }
        Ok(ItemPedido {
            produto: produto.to_string(),
            quantidade,
            preco_unitario,
        })
    }

    pub fn produto(&self) -> &str {
        &self.produto
    }

    pub fn subtotal(&self) -> f64 {
        self.quantidade as f64 * self.preco_unitario
    }
}

pub struct Pedido {
    itens: Vec<ItemPedido>,
}

impl Pedido {
    pub fn novo() -> Self {
        Pedido { itens: Vec::new() }
    }

    pub fn adicionar_item(&mut self, item: ItemPedido) {
        self.itens.push(item);
    }

    pub fn total(&self) -> f64 {
        self.itens.iter().map(|item| item.subtotal()).sum()
    }

    pub fn quantidade_itens(&self) -> usize {
        self.itens.len()
    }
}

// Porta definida pelo dominio; a implementacao concreta mora na infraestrutura.
pub trait PedidoRepository {
    fn salvar(&mut self, pedido: Pedido);
    fn buscar_todos(&self) -> &Vec<Pedido>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_invalido_falha_na_criacao() {
        assert!(ItemPedido::new("Mouse", 0, 50.0).is_err());
        assert!(ItemPedido::new("Mouse", 1, 0.0).is_err());
    }

    #[test]
    fn item_valido_expoe_seus_dados() {
        let item = ItemPedido::new("Mouse", 2, 50.0).unwrap();
        assert_eq!(item.produto(), "Mouse");
        assert_eq!(item.subtotal(), 100.0);
    }

    #[test]
    fn pedido_calcula_total_corretamente() {
        let mut pedido = Pedido::novo();
        let item1 = ItemPedido::new("Mouse", 2, 50.0).unwrap();
        let item2 = ItemPedido::new("Teclado", 1, 150.0).unwrap();

        pedido.adicionar_item(item1);
        pedido.adicionar_item(item2);

        assert_eq!(pedido.quantidade_itens(), 2);
        assert_eq!(pedido.total(), 250.0);
    }
}
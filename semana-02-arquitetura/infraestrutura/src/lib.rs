use dominio_pedidos::{Pedido, PedidoRepository};

pub struct RepositorioEmMemoria {
    pedidos: Vec<Pedido>,
}

impl RepositorioEmMemoria {
    pub fn novo() -> Self {
        RepositorioEmMemoria { pedidos: Vec::new() }
    }
}

impl PedidoRepository for RepositorioEmMemoria {
    fn salvar(&mut self, pedido: Pedido) {
        self.pedidos.push(pedido);
    }

    fn buscar_todos(&self) -> &Vec<Pedido> {
        &self.pedidos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dominio_pedidos::ItemPedido;

    #[test]
    fn repositorio_salva_e_recupera_pedidos() {
        let mut repositorio = RepositorioEmMemoria::novo();

        let mut pedido = Pedido::novo();
        pedido.adicionar_item(ItemPedido::new("Monitor", 1, 900.0).unwrap());

        repositorio.salvar(pedido);

        assert_eq!(repositorio.buscar_todos().len(), 1);
        assert_eq!(repositorio.buscar_todos()[0].total(), 900.0);
    }
}
use dominio_pedidos::{ItemPedido, Pedido, PedidoRepository};
use std::sync::{Arc, Mutex};

pub struct ItemComando {
    pub produto: String,
    pub quantidade: u32,
    pub preco_unitario: f64,
}

pub struct CriarPedidoComando {
    pub itens: Vec<ItemComando>,
}

pub struct CriarPedidoCasoDeUso {
    repositorio: Arc<Mutex<dyn PedidoRepository + Send>>,
}

impl CriarPedidoCasoDeUso {
    pub fn novo(repositorio: Arc<Mutex<dyn PedidoRepository + Send>>) -> Self {
        CriarPedidoCasoDeUso { repositorio }
    }

    pub fn executar(&self, comando: CriarPedidoComando) -> Result<f64, String> {
        let mut pedido = Pedido::novo();

        for item in comando.itens {
            let item_pedido = ItemPedido::new(&item.produto, item.quantidade, item.preco_unitario)?;
            pedido.adicionar_item(item_pedido);
        }

        let total = pedido.total();

        let mut repositorio = self.repositorio.lock().unwrap();
        repositorio.salvar(pedido);

        Ok(total)
    }
}

pub struct ListarPedidosCasoDeUso {
    repositorio: Arc<Mutex<dyn PedidoRepository + Send>>,
}

impl ListarPedidosCasoDeUso {
    pub fn novo(repositorio: Arc<Mutex<dyn PedidoRepository + Send>>) -> Self {
        ListarPedidosCasoDeUso { repositorio }
    }

    pub fn executar(&self) -> Vec<f64> {
        let repositorio = self.repositorio.lock().unwrap();
        repositorio.buscar_todos().iter().map(|p| p.total()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RepositorioFalso {
        pedidos: Vec<Pedido>,
    }

    impl RepositorioFalso {
        fn novo() -> Self {
            RepositorioFalso { pedidos: Vec::new() }
        }
    }

    impl PedidoRepository for RepositorioFalso {
        fn salvar(&mut self, pedido: Pedido) {
            self.pedidos.push(pedido);
        }

        fn buscar_todos(&self) -> &Vec<Pedido> {
            &self.pedidos
        }
    }

    #[test]
    fn criar_pedido_salva_e_retorna_total() {
        let repositorio: Arc<Mutex<dyn PedidoRepository + Send>> =
            Arc::new(Mutex::new(RepositorioFalso::novo()));
        let caso_de_uso = CriarPedidoCasoDeUso::novo(Arc::clone(&repositorio));

        let comando = CriarPedidoComando {
            itens: vec![ItemComando {
                produto: "Mouse".to_string(),
                quantidade: 2,
                preco_unitario: 50.0,
            }],
        };

        let total = caso_de_uso.executar(comando).unwrap();

        assert_eq!(total, 100.0);
        assert_eq!(repositorio.lock().unwrap().buscar_todos().len(), 1);
    }

    #[test]
    fn listar_pedidos_retorna_totais() {
        let repositorio: Arc<Mutex<dyn PedidoRepository + Send>> =
            Arc::new(Mutex::new(RepositorioFalso::novo()));
        let criar = CriarPedidoCasoDeUso::novo(Arc::clone(&repositorio));

        criar
            .executar(CriarPedidoComando {
                itens: vec![ItemComando {
                    produto: "Item".to_string(),
                    quantidade: 1,
                    preco_unitario: 250.0,
                }],
            })
            .unwrap();

        let listar = ListarPedidosCasoDeUso::novo(Arc::clone(&repositorio));
        let totais = listar.executar();

        assert_eq!(totais, vec![250.0]);
    }
}
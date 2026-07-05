use dominio_clientes::Cliente;
use dominio_pedidos::Pedido;

pub struct PoliticaCredito {
    limite: f64,
}

impl PoliticaCredito {
    pub fn nova(limite: f64) -> Self {
        PoliticaCredito { limite }
    }

    pub fn aprovar_pedido(
        &self,
        _cliente: &Cliente,
        pedidos_anteriores: &[Pedido],
        novo_pedido: &Pedido,
    ) -> Result<(), String> {
        let total_anterior: f64 = pedidos_anteriores.iter().map(|p| p.total()).sum();
        let total_com_novo = total_anterior + novo_pedido.total();

        if total_com_novo > self.limite {
            Err(format!(
                "limite de credito excedido: total {total_com_novo:.2} ultrapassa o limite de {:.2}",
                self.limite
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dominio_pedidos::ItemPedido;

    fn criar_pedido(total: f64) -> Pedido {
        let mut pedido = Pedido::novo();
        pedido.adicionar_item(ItemPedido::new("Item generico", 1, total).unwrap());
        pedido
    }

    #[test]
    fn aprova_pedido_dentro_do_limite() {
        let cliente = Cliente::novo("Rodrigo", "rodrigo@exemplo.com", "12345678901").unwrap();
        let politica = PoliticaCredito::nova(5000.0);

        let pedidos_anteriores = vec![criar_pedido(1000.0), criar_pedido(2000.0)];
        let novo_pedido = criar_pedido(1500.0);

        let resultado = politica.aprovar_pedido(&cliente, &pedidos_anteriores, &novo_pedido);

        assert!(resultado.is_ok());
    }

    #[test]
    fn rejeita_pedido_que_excede_limite() {
        let cliente = Cliente::novo("Rodrigo", "rodrigo@exemplo.com", "12345678901").unwrap();
        let politica = PoliticaCredito::nova(5000.0);

        let pedidos_anteriores = vec![criar_pedido(3000.0)];
        let novo_pedido = criar_pedido(2500.0);

        let resultado = politica.aprovar_pedido(&cliente, &pedidos_anteriores, &novo_pedido);

        assert!(resultado.is_err());
    }
}
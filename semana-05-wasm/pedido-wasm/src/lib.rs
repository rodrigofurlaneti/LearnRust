use dominio_pedidos::{ItemPedido, Pedido};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn calcular_total_pedido(quantidade: u32, preco_unitario: f64) -> Result<f64, JsValue> {
    let item = ItemPedido::new("item", quantidade, preco_unitario)
        .map_err(|erro| JsValue::from_str(&erro))?;

    let mut pedido = Pedido::novo();
    pedido.adicionar_item(item);

    Ok(pedido.total())
}
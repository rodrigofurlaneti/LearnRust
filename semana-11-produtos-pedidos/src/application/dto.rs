use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// DTOs de entrada/saida dos casos de uso. Ficam na aplicacao (nao na
/// presentation) porque representam a "linguagem" dos casos de uso, e a
/// presentation apenas (de)serializa isso de/para JSON. Mesmo trade-off
/// documentado na Semana 9: `ToSchema` (Swagger) e derivado aqui junto
/// com `Serialize`/`Deserialize`.

// ---------------------------------------------------------------------
// Usuario (auth) - herdado da Semana 9, com UpdateUser/DeleteUser novos.
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterUserInput {
    #[schema(example = "rodrigo@example.com")]
    pub email: String,
    #[schema(example = "SenhaForte123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterUserOutput {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthenticateUserInput {
    #[schema(example = "rodrigo@example.com")]
    pub email: String,
    #[schema(example = "SenhaForte123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthenticateUserOutput {
    pub access_token: String,
}

/// Saida de `GET /me` - dados do usuario associado ao token enviado.
#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserOutput {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

/// Entrada de `PUT /admin/usuarios/:id` (novo na Semana 10, restrito a
/// `AdminUser`). O id vem da URL, nao do corpo - so o email e editavel
/// por este caso de uso.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserInput {
    #[schema(example = "novo-email@example.com")]
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateUserOutput {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

// ---------------------------------------------------------------------
// Cliente (novo na Semana 10).
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterClienteInput {
    #[schema(example = "Rodrigo Furlaneti")]
    pub nome: String,
    #[schema(example = "111.444.777-35")]
    pub documento: String,
    #[schema(example = "cliente@example.com")]
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateClienteInput {
    #[schema(example = "Rodrigo Furlaneti")]
    pub nome: String,
    #[schema(example = "111.444.777-35")]
    pub documento: String,
    #[schema(example = "cliente@example.com")]
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClienteOutput {
    pub cliente_id: String,
    pub nome: String,
    pub documento: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListClientesOutput {
    pub clientes: Vec<ClienteOutput>,
}

// ---------------------------------------------------------------------
// Produto (novo na Semana 11).
// ---------------------------------------------------------------------

/// `preco` chega como texto (ex.: "19.90"), nunca como numero de ponto
/// flutuante - o mesmo motivo pelo qual `domain::Dinheiro` existe (ver
/// comentario la). Um `f64` no JSON ja teria perdido precisao antes
/// mesmo de chegar no dominio.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterProdutoInput {
    #[schema(example = "Teclado mecanico")]
    pub nome: String,
    #[schema(example = "349.90")]
    pub preco: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProdutoInput {
    #[schema(example = "Teclado mecanico")]
    pub nome: String,
    #[schema(example = "349.90")]
    pub preco: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProdutoOutput {
    pub produto_id: String,
    pub nome: String,
    pub preco: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListProdutosOutput {
    pub produtos: Vec<ProdutoOutput>,
}

// ---------------------------------------------------------------------
// Pedido (novo na Semana 11).
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct ItemPedidoInput {
    pub produto_id: String,
    #[schema(example = 2)]
    pub quantidade: u32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterPedidoInput {
    pub cliente_id: String,
    pub itens: Vec<ItemPedidoInput>,
}

/// `valor_unitario`/`valor_total` sao o snapshot congelado no momento do
/// pedido (ver `domain::ItemPedido`) - nunca o preco atual do produto.
#[derive(Debug, Serialize, ToSchema)]
pub struct ItemPedidoOutput {
    pub produto_id: String,
    pub nome_produto: String,
    pub quantidade: u32,
    pub valor_unitario: String,
    pub valor_total: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PedidoOutput {
    pub pedido_id: String,
    pub cliente_id: String,
    pub itens: Vec<ItemPedidoOutput>,
    pub valor_total: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListPedidosOutput {
    pub pedidos: Vec<PedidoOutput>,
}

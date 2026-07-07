use std::sync::Arc;

use crate::application::authenticate_user::AuthenticateUser;
use crate::application::delete_cliente::DeleteCliente;
use crate::application::delete_pedido::DeletePedido;
use crate::application::delete_produto::DeleteProduto;
use crate::application::delete_user::DeleteUser;
use crate::application::get_cliente::GetCliente;
use crate::application::get_current_user::GetCurrentUser;
use crate::application::get_pedido::GetPedido;
use crate::application::get_produto::GetProduto;
use crate::application::list_clientes::ListClientes;
use crate::application::list_pedidos::ListPedidos;
use crate::application::list_produtos::ListProdutos;
use crate::application::register_cliente::RegisterCliente;
use crate::application::register_pedido::RegisterPedido;
use crate::application::register_produto::RegisterProduto;
use crate::application::register_user::RegisterUser;
use crate::application::token_service::TokenService;
use crate::application::update_cliente::UpdateCliente;
use crate::application::update_produto::UpdateProduto;
use crate::application::update_user::UpdateUser;

/// Estado compartilhado do Axum. E aqui, e so aqui, que a presentation
/// conhece os casos de uso concretos - ela nunca conhece repositorio,
/// hasher, cache ou auditoria diretamente (isso e resolvido no
/// `main.rs`). `tokens` fica exposto tambem aqui porque o extractor
/// `AuthenticatedUser` precisa verificar tokens em toda rota protegida,
/// nao so no login.
#[derive(Clone)]
pub struct AppState {
    pub register_user: Arc<RegisterUser>,
    pub authenticate_user: Arc<AuthenticateUser>,
    pub get_current_user: Arc<GetCurrentUser>,
    pub update_user: Arc<UpdateUser>,
    pub delete_user: Arc<DeleteUser>,
    pub register_cliente: Arc<RegisterCliente>,
    pub get_cliente: Arc<GetCliente>,
    pub list_clientes: Arc<ListClientes>,
    pub update_cliente: Arc<UpdateCliente>,
    pub delete_cliente: Arc<DeleteCliente>,
    pub register_produto: Arc<RegisterProduto>,
    pub get_produto: Arc<GetProduto>,
    pub list_produtos: Arc<ListProdutos>,
    pub update_produto: Arc<UpdateProduto>,
    pub delete_produto: Arc<DeleteProduto>,
    pub register_pedido: Arc<RegisterPedido>,
    pub get_pedido: Arc<GetPedido>,
    pub list_pedidos: Arc<ListPedidos>,
    pub delete_pedido: Arc<DeletePedido>,
    pub tokens: Arc<dyn TokenService>,
}

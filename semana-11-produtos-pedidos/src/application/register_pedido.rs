use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::{ItemPedidoInput, ItemPedidoOutput, PedidoOutput, RegisterPedidoInput};
use crate::application::errors::ApplicationError;
use crate::domain::{
    ClienteId, ClienteRepository, DomainError, ItemPedido, Pedido, PedidoRepository, ProdutoId,
    ProdutoRepository, Quantidade,
};

/// Caso de uso de criacao de pedido - o coracao da Semana 11. Depende de
/// TRES portas (`ClienteRepository`, `ProdutoRepository`,
/// `PedidoRepository`) porque um pedido, por natureza, coordena tres
/// agregados diferentes - o mesmo papel que `PoliticaCredito` (Domain
/// Service da Semana 2, `dominio-vendas`) ja cumpria coordenando Cliente
/// e Pedido sem pertencer a nenhum dos dois. Aqui a coordenacao acontece
/// na camada de aplicacao (um caso de uso), nao num Domain Service, porque
/// a regra em si ("copiar preco atual para dentro do pedido") e simples o
/// bastante para nao precisar de um objeto de dominio proprio.
pub struct RegisterPedido {
    cliente_repository: Arc<dyn ClienteRepository>,
    produto_repository: Arc<dyn ProdutoRepository>,
    pedido_repository: Arc<dyn PedidoRepository>,
}

impl RegisterPedido {
    pub fn new(
        cliente_repository: Arc<dyn ClienteRepository>,
        produto_repository: Arc<dyn ProdutoRepository>,
        pedido_repository: Arc<dyn PedidoRepository>,
    ) -> Self {
        Self {
            cliente_repository,
            produto_repository,
            pedido_repository,
        }
    }

    pub async fn execute(&self, input: RegisterPedidoInput) -> Result<PedidoOutput, ApplicationError> {
        let cliente_id = Self::parse_cliente_id(&input.cliente_id)?;
        self.ensure_cliente_existe(cliente_id).await?;

        let itens = self.montar_itens_com_preco_atual(input.itens).await?;
        let pedido = Pedido::register(cliente_id, itens)?;
        self.pedido_repository.save(&pedido).await?;

        Ok(Self::to_output(&pedido))
    }

    async fn ensure_cliente_existe(&self, cliente_id: ClienteId) -> Result<(), ApplicationError> {
        let existente = self.cliente_repository.find_by_id(cliente_id).await?;
        existente
            .map(|_| ())
            .ok_or(ApplicationError::Domain(DomainError::ClienteNotFound))
    }

    /// Para cada linha do pedido, busca o produto no repositorio e
    /// congela nome/preco ATUAIS dentro do `ItemPedido` - e aqui que a
    /// regra de negocio "salvar o valor unitario e o valor total no
    /// momento do pedido" e aplicada.
    async fn montar_itens_com_preco_atual(
        &self,
        entradas: Vec<ItemPedidoInput>,
    ) -> Result<Vec<ItemPedido>, ApplicationError> {
        let mut itens = Vec::with_capacity(entradas.len());
        for entrada in entradas {
            itens.push(self.montar_item(entrada).await?);
        }
        Ok(itens)
    }

    async fn montar_item(&self, entrada: ItemPedidoInput) -> Result<ItemPedido, ApplicationError> {
        let produto_id = Self::parse_produto_id(&entrada.produto_id)?;
        let quantidade = Quantidade::parse(entrada.quantidade)?;
        let produto = self
            .produto_repository
            .find_by_id(produto_id)
            .await?
            .ok_or(ApplicationError::Domain(DomainError::ProdutoNotFound))?;

        Ok(ItemPedido::snapshot_no_momento_do_pedido(
            produto.id(),
            produto.nome().clone(),
            quantidade,
            produto.preco(),
        ))
    }

    fn parse_cliente_id(raw: &str) -> Result<ClienteId, ApplicationError> {
        Uuid::parse_str(raw)
            .map(ClienteId::from_uuid)
            .map_err(|_| ApplicationError::Domain(DomainError::InvalidId))
    }

    fn parse_produto_id(raw: &str) -> Result<ProdutoId, ApplicationError> {
        Uuid::parse_str(raw)
            .map(ProdutoId::from_uuid)
            .map_err(|_| ApplicationError::Domain(DomainError::InvalidId))
    }

    pub(crate) fn to_output(pedido: &Pedido) -> PedidoOutput {
        let itens = pedido.itens().iter().map(Self::item_to_output).collect();

        PedidoOutput {
            pedido_id: pedido.id().as_uuid().to_string(),
            cliente_id: pedido.cliente_id().as_uuid().to_string(),
            itens,
            valor_total: pedido.valor_total().as_reais_str(),
        }
    }

    fn item_to_output(item: &ItemPedido) -> ItemPedidoOutput {
        ItemPedidoOutput {
            produto_id: item.produto_id().as_uuid().to_string(),
            nome_produto: item.nome_produto().as_str().to_string(),
            quantidade: item.quantidade().valor(),
            valor_unitario: item.valor_unitario().as_reais_str(),
            valor_total: item.valor_total().as_reais_str(),
        }
    }
}

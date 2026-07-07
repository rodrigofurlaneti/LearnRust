use std::sync::Arc;

use crate::application::shared::dto::{ProdutoOutput, RegisterProdutoInput};
use crate::application::shared::errors::ApplicationError;
use crate::domain::{Dinheiro, Nome, Produto, ProdutoRepository};

/// Caso de uso de cadastro de produto (novo na Semana 11), simetrico ao
/// `RegisterCliente`: uma unica responsabilidade (SRP), depende so da
/// abstracao `ProdutoRepository` (DIP).
pub struct RegisterProduto {
    repository: Arc<dyn ProdutoRepository>,
}

impl RegisterProduto {
    pub fn new(repository: Arc<dyn ProdutoRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: RegisterProdutoInput,
    ) -> Result<ProdutoOutput, ApplicationError> {
        let nome = Nome::parse(&input.nome)?;
        let preco = Dinheiro::parse_reais(&input.preco)?;

        let produto = Produto::register(nome, preco);
        self.repository.save(&produto).await?;

        Ok(Self::to_output(&produto))
    }

    pub(crate) fn to_output(produto: &Produto) -> ProdutoOutput {
        ProdutoOutput {
            produto_id: produto.id().as_uuid().to_string(),
            nome: produto.nome().as_str().to_string(),
            preco: produto.preco().as_reais_str(),
        }
    }
}

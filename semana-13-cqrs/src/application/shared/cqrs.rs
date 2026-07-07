//! Vocabulario minimo do CQRS (Command Query Responsibility Segregation)
//! "leve": Command e Query sao marcadores que so declaram o tipo de saida
//! da operacao; CommandHandler/QueryHandler sao quem sabe executa-la.
//!
//! Este projeto NAO separa banco de escrita e banco de leitura (isso
//! seria CQRS "completo", com Read Models proprios, projetados a partir
//! dos mesmos eventos que hoje viram `auditoria.txt` - ver README para o
//! racional). Aqui o objetivo e mais modesto: dar nomes explicitos
//! (`Command`, `Query`, `Handler`, `Bus`) para uma separacao que o codigo
//! ja tinha implicitamente desde a Semana 9 (`register_*`/`update_*`/
//! `delete_*` sempre mudam estado, `get_*`/`list_*` sempre so leem), e
//! trocar "a presentation conhece 19 casos de uso" por "a presentation
//! conhece 2 barramentos".

use async_trait::async_trait;

use crate::application::shared::errors::ApplicationError;

/// Uma intencao de MUDAR o estado do sistema. Nunca serve dados de
/// consulta alem do minimo necessario para confirmar que a operacao
/// aconteceu (o agregado criado/atualizado, ou nada no caso de remocao).
pub trait Command {
    type Output;
}

/// Uma pergunta sobre o estado ATUAL do sistema. Nunca muda estado -
/// rodar a mesma Query duas vezes seguidas nunca produz efeito colateral
/// (idempotente por natureza, ao contrario de um Command).
pub trait Query {
    type Output;
}

/// Quem sabe executar um Command especifico. Cada caso de uso de escrita
/// existente (`RegisterCliente`, `UpdateCliente` etc.) implementa este
/// trait para o Command correspondente, delegando para o proprio
/// `execute` que ja existia antes desta semana - nenhuma regra de negocio
/// muda, so ganha um nome (`Command`) e uma porta de entrada uniforme
/// (`handle`).
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Output, ApplicationError>;
}

/// Quem sabe executar uma Query especifica. Mesma ideia do
/// `CommandHandler`, para o lado de leitura.
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Output, ApplicationError>;
}

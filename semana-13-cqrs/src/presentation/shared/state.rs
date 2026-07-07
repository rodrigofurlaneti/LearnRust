use std::sync::Arc;

use crate::application::shared::command_bus::CommandBus;
use crate::application::shared::query_bus::QueryBus;
use crate::application::shared::token_service::TokenService;

/// Estado compartilhado do Axum. A Semana 13 troca 19 campos
/// `Arc<UseCase>` por 2: todo Command passa pelo `CommandBus`, toda Query
/// pelo `QueryBus`, e a presentation deixou de precisar conhecer cada
/// caso de uso individualmente. `tokens` continua exposto direto, fora
/// dos barramentos, porque o extractor `AuthenticatedUser` so verifica um
/// JWT: nao muda estado (nao e um Command) nem consulta um agregado (nao
/// e uma Query de negocio), e sim uma checagem criptografica de borda que
/// roda antes mesmo do handler decidir se vai despachar algo.
#[derive(Clone)]
pub struct AppState {
    pub command_bus: Arc<CommandBus>,
    pub query_bus: Arc<QueryBus>,
    pub tokens: Arc<dyn TokenService>,
}

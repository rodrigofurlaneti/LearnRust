use crate::domain::UserId;

/// Value Object simples para não passar `String` crua de token pelo sistema.
#[derive(Debug, Clone)]
pub struct AccessToken(pub String);

/// Porta de saída: a aplicação só sabe que consegue "emitir um token para um
/// usuário". Não sabe se por trás disso tem JWT, sessão opaca, etc.
pub trait TokenService: Send + Sync {
    fn issue(&self, user_id: UserId) -> Result<AccessToken, String>;
}

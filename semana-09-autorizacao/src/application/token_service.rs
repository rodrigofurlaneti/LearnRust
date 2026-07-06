use crate::domain::{Role, UserId};

/// Value Object simples para nao passar `String` crua de token pelo sistema.
#[derive(Debug, Clone)]
pub struct AccessToken(pub String);

/// Claims decodificadas de um token valido - o que a aplicacao precisa
/// saber sobre quem fez a requisicao, sem conhecer o formato do token.
#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: UserId,
    pub role: Role,
}

/// Porta de saida: a aplicacao (e o extractor de autenticacao da
/// presentation) so sabem que conseguem "emitir um token para um usuario"
/// e "verificar se um token e valido". Nao sabem se por tras disso tem
/// JWT, sessao opaca, etc.
pub trait TokenService: Send + Sync {
    fn issue(&self, user_id: UserId, role: Role) -> Result<AccessToken, String>;
    fn verify(&self, token: &str) -> Result<TokenClaims, String>;
}

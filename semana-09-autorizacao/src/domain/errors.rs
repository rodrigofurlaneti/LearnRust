use thiserror::Error;

/// Erros que representam violacoes das regras de negocio do dominio.
/// Nenhuma camada externa (infra/presentation) deve inventar seus proprios
/// codigos de erro de negocio: eles nascem aqui.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("email invalido")]
    InvalidEmail,

    #[error("senha fraca: use ao menos 8 caracteres, com maiuscula, minuscula e numero")]
    WeakPassword,

    #[error("ja existe um usuario cadastrado com este email")]
    UserAlreadyExists,

    #[error("credenciais invalidas")]
    InvalidCredentials,

    #[error("voce nao tem permissao para acessar este recurso")]
    PermissionDenied,
}

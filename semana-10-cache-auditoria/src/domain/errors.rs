use thiserror::Error;

/// Erros que representam violacoes das regras de negocio do dominio.
/// Nenhuma camada externa (infra/presentation) deve inventar seus proprios
/// codigos de erro de negocio: eles nascem aqui. Herdado da Semana 9 e
/// ampliado com o vocabulario de erro de Usuario (update/delete) e de
/// Cliente (cadastro novo desta semana).
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

    #[error("usuario nao encontrado")]
    UserNotFound,

    #[error("nome de cliente invalido: informe ao menos 2 caracteres")]
    InvalidClienteName,

    #[error("documento (CPF) invalido")]
    InvalidDocument,

    #[error("ja existe um cliente cadastrado com este documento")]
    ClienteAlreadyExists,

    #[error("cliente nao encontrado")]
    ClienteNotFound,

    #[error("identificador invalido: deve ser um UUID")]
    InvalidId,
}

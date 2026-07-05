use thiserror::Error;

/// Erros que representam violações das regras de negócio do domínio.
/// Nenhuma camada externa (infra/presentation) deve inventar seus próprios
/// códigos de erro de negócio: eles nascem aqui.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("email inválido")]
    InvalidEmail,

    #[error("senha fraca: use ao menos 8 caracteres, com maiúscula, minúscula e número")]
    WeakPassword,

    #[error("já existe um usuário cadastrado com este email")]
    UserAlreadyExists,

    #[error("credenciais inválidas")]
    InvalidCredentials,
}

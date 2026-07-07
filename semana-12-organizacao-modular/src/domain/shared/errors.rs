use thiserror::Error;

/// Erros que representam violacoes das regras de negocio do dominio.
/// Nenhuma camada externa (infra/presentation) deve inventar seus proprios
/// codigos de erro de negocio: eles nascem aqui. Crescendo desde a
/// Semana 9: RBAC, depois Cliente (Semana 10), agora Produto/Pedido
/// (Semana 11).
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

    #[error("nome invalido: informe ao menos 2 caracteres")]
    InvalidNome,

    #[error("documento (CPF) invalido")]
    InvalidDocument,

    #[error("ja existe um cliente cadastrado com este documento")]
    ClienteAlreadyExists,

    #[error("cliente nao encontrado")]
    ClienteNotFound,

    #[error("identificador invalido: deve ser um UUID")]
    InvalidId,

    #[error("valor monetario invalido: deve ser um numero decimal maior ou igual a zero")]
    ValorMonetarioInvalido,

    #[error("quantidade invalida: deve ser maior que zero")]
    InvalidQuantidade,

    #[error("produto nao encontrado")]
    ProdutoNotFound,

    #[error("ja existe um produto cadastrado com este identificador")]
    ProdutoAlreadyExists,

    #[error("pedido nao encontrado")]
    PedidoNotFound,

    #[error("ja existe um pedido cadastrado com este identificador")]
    PedidoAlreadyExists,

    #[error("pedido precisa ter pelo menos um item")]
    PedidoSemItens,
}

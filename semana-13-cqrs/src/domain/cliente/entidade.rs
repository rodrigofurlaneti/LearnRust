use crate::domain::cliente::cliente_id::ClienteId;
use crate::domain::cliente::cpf::Cpf;
use crate::domain::shared::email::Email;
use crate::domain::shared::nome::Nome;

/// Entidade raiz de agregado (novo na Semana 10): representa o cadastro de
/// cliente, um contexto de negocio separado do cadastro de usuario
/// (`User`) - um cliente nao faz login, nao tem senha nem papel. Os dois
/// agregados compartilham apenas o Value Object `Email` (o conceito
/// "email valido" e identico nos dois contextos).
///
/// Mesma nota de Object Calisthenics de `User`: quatro campos, cada um um
/// Value Object coeso - mantemos essa cardinalidade por simetria e porque
/// quebrar em wrappers artificiais so pioraria a legibilidade.
#[derive(Clone)]
pub struct Cliente {
    id: ClienteId,
    nome: Nome,
    documento: Cpf,
    email: Email,
}

impl Cliente {
    /// Fluxo publico de cadastro (`POST /clientes`).
    pub fn register(nome: Nome, documento: Cpf, email: Email) -> Self {
        Self {
            id: ClienteId::new(),
            nome,
            documento,
            email,
        }
    }

    /// Reidrata um cliente ja existente a partir dos dados guardados no
    /// cache. So a camada de infraestrutura deve chamar isto.
    pub fn reconstitute(id: ClienteId, nome: Nome, documento: Cpf, email: Email) -> Self {
        Self {
            id,
            nome,
            documento,
            email,
        }
    }

    /// Regra de negocio de atualizacao cadastral (`UpdateCliente`): troca
    /// nome/documento/email mantendo a identidade. Devolve um novo
    /// `Cliente` - o agregado continua imutavel de fora, quem chama e quem
    /// decide persistir o resultado.
    pub fn with_updated_data(&self, nome: Nome, documento: Cpf, email: Email) -> Self {
        Self {
            id: self.id,
            nome,
            documento,
            email,
        }
    }

    pub fn id(&self) -> ClienteId {
        self.id
    }

    pub fn nome(&self) -> &Nome {
        &self.nome
    }

    pub fn documento(&self) -> &Cpf {
        &self.documento
    }

    pub fn email(&self) -> &Email {
        &self.email
    }
}

use crate::domain::email::Email;
use crate::domain::password::{HashedPassword, PlainPassword};
use crate::domain::password_hasher::PasswordHasher;
use crate::domain::user_id::UserId;

/// Entidade raiz de agregado. Concentra a regra de negócio de autenticação:
/// só o próprio `User` sabe decidir se uma senha candidata é a sua senha
/// ("Tell, Don't Ask" — em vez de expor o hash para alguém comparar de fora).
///
/// Nota sobre Object Calisthenics: a regra "no máximo 2 variáveis de
/// instância por classe" é a mais extrema das nove e, aplicada ao pé da
/// letra, obrigaria a quebrar `id/email/password` em objetos-envelope
/// artificiais. Preferimos manter os três campos, já que cada um é um Value
/// Object coeso — a intenção da regra (evitar entidades inchadas) já está
/// satisfeita.
pub struct User {
    id: UserId,
    email: Email,
    password: HashedPassword,
}

impl User {
    /// Cria um novo usuário (fluxo de cadastro). Gera um novo `UserId`.
    pub fn register(email: Email, password: HashedPassword) -> Self {
        Self {
            id: UserId::new(),
            email,
            password,
        }
    }

    /// Reidrata um usuário já existente a partir dos dados persistidos.
    /// Só a camada de infraestrutura deve chamar isto.
    pub fn reconstitute(id: UserId, email: Email, password: HashedPassword) -> Self {
        Self { id, email, password }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> &HashedPassword {
        &self.password
    }

    /// Regra de negócio: um usuário só "bate" com uma senha se o hasher
    /// confirmar a correspondência. O domínio não sabe *como* isso é feito.
    pub async fn matches_password(
        &self,
        candidate: &PlainPassword,
        hasher: &dyn PasswordHasher,
    ) -> bool {
        hasher.verify(candidate, &self.password).await
    }
}

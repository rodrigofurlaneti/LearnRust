use crate::domain::email::Email;
use crate::domain::password::{HashedPassword, PlainPassword};
use crate::domain::password_hasher::PasswordHasher;
use crate::domain::role::Role;
use crate::domain::user_id::UserId;

/// Entidade raiz de agregado. Concentra a regra de negocio de autenticacao:
/// so o proprio `User` sabe decidir se uma senha candidata e a sua senha
/// ("Tell, Don't Ask" - em vez de expor o hash para alguem comparar de fora).
///
/// Nota sobre Object Calisthenics: a regra "no maximo 2 variaveis de
/// instancia por classe" e a mais extrema das nove e, aplicada ao pe da
/// letra, obrigaria a quebrar `id/email/password/role` em objetos-envelope
/// artificiais. Preferimos manter os quatro campos, ja que cada um e um
/// Value Object coeso - a intencao da regra (evitar entidades inchadas) ja
/// esta satisfeita.
pub struct User {
    id: UserId,
    email: Email,
    password: HashedPassword,
    role: Role,
}

impl User {
    /// Fluxo publico de auto-cadastro (`POST /auth/register`). Sempre cria
    /// um usuario comum - de proposito nao aceita `Role` como parametro,
    /// para que seja impossivel qualquer pessoa se auto-promover a admin
    /// so chamando o endpoint publico.
    pub fn register(email: Email, password: HashedPassword) -> Self {
        Self::register_with_role(email, password, Role::User)
    }

    /// Fluxo interno/administrativo (seed, seed de admin, futura tela de
    /// gestao de usuarios). Fica explicito no nome que quem chama esta
    /// decidindo conscientemente o papel do usuario.
    pub fn register_with_role(email: Email, password: HashedPassword, role: Role) -> Self {
        Self {
            id: UserId::new(),
            email,
            password,
            role,
        }
    }

    /// Reidrata um usuario ja existente a partir dos dados persistidos.
    /// So a camada de infraestrutura deve chamar isto.
    pub fn reconstitute(id: UserId, email: Email, password: HashedPassword, role: Role) -> Self {
        Self {
            id,
            email,
            password,
            role,
        }
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

    pub fn role(&self) -> Role {
        self.role
    }

    /// Regra de negocio: um usuario so "bate" com uma senha se o hasher
    /// confirmar a correspondencia. O dominio nao sabe *como* isso e feito.
    pub async fn matches_password(
        &self,
        candidate: &PlainPassword,
        hasher: &dyn PasswordHasher,
    ) -> bool {
        hasher.verify(candidate, &self.password).await
    }
}

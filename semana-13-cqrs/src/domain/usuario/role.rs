/// Papel do usuario no sistema. Enum fechado (nao uma `String` solta) -
/// mesma filosofia de "wrap primitives" aplicada aos Value Objects das
/// semanas anteriores. Usado tanto para persistencia quanto como claim no
/// JWT, e conferido pelo guard de RBAC na camada de presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Admin => "admin",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "user" => Some(Role::User),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

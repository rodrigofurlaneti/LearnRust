use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::application::shared::errors::ApplicationError;
use crate::domain::{DomainError, Role, UserId};
use crate::presentation::shared::error_response::ApiError;
use crate::presentation::shared::state::AppState;

/// Extractor Axum: qualquer handler que declare `AuthenticatedUser` como
/// parametro so executa se o header `Authorization: Bearer <token>`
/// trouxer um JWT valido. Sem token valido, a requisicao nem chega no
/// handler - o Axum ja responde 401 antes disso (via `Rejection`).
/// Herdado sem alteracoes da Semana 9 - a Semana 10 reaproveita a mesma
/// estrutura de autenticacao/autorizacao, so trocando o que ha "por
/// baixo" (repositorio de usuarios agora e cache + auditoria).
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub role: Role,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = Self::extract_bearer_token(parts)?;
        let claims = state.tokens.verify(&token).map_err(|_| Self::unauthorized())?;

        Ok(Self {
            user_id: claims.user_id,
            role: claims.role,
        })
    }
}

impl AuthenticatedUser {
    fn extract_bearer_token(parts: &Parts) -> Result<String, ApiError> {
        let header_value = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(Self::unauthorized)?;

        header_value
            .strip_prefix("Bearer ")
            .map(str::to_string)
            .ok_or_else(Self::unauthorized)
    }

    fn unauthorized() -> ApiError {
        ApiError::from(ApplicationError::Domain(DomainError::InvalidCredentials))
    }
}

/// Guard adicional de RBAC: so libera a requisicao se o usuario
/// autenticado tiver o papel Admin. Composicao sobre `AuthenticatedUser`,
/// sem duplicar a logica de validacao do token. Reaproveitado nesta
/// semana para proteger a gestao administrativa de usuarios
/// (`PUT`/`DELETE /admin/usuarios/:id`) e a remocao de clientes
/// (`DELETE /clientes/:id`).
pub struct AdminUser(pub AuthenticatedUser);

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if user.role != Role::Admin {
            return Err(ApiError::from(ApplicationError::Domain(
                DomainError::PermissionDenied,
            )));
        }

        Ok(Self(user))
    }
}

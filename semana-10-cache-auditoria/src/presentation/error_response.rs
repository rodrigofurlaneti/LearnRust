use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::application::errors::ApplicationError;
use crate::domain::DomainError;

/// Formato do corpo de erro devolvido pela API. Existe como struct (em vez
/// de so o `json!(...)` solto) para poder ser referenciado nas respostas
/// documentadas pelo Swagger.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

/// Traducao de erros de aplicacao/dominio para respostas HTTP.
/// Nenhuma outra camada sabe o que e um `StatusCode` - isso e vocabulario
/// exclusivo da presentation. Ampliado nesta semana com o vocabulario de
/// erro de `Cliente` e de gestao de `User` (update/delete).
pub struct ApiError(ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        Self(error)
    }
}

impl ApiError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match &self.0 {
            ApplicationError::Domain(domain_error) => Self::status_for_domain_error(domain_error),
            ApplicationError::Unexpected(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
        }
    }

    fn status_for_domain_error(domain_error: &DomainError) -> (StatusCode, String) {
        let status = match domain_error {
            DomainError::InvalidEmail
            | DomainError::WeakPassword
            | DomainError::InvalidClienteName
            | DomainError::InvalidDocument
            | DomainError::InvalidId => StatusCode::BAD_REQUEST,
            DomainError::UserAlreadyExists | DomainError::ClienteAlreadyExists => {
                StatusCode::CONFLICT
            }
            DomainError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            DomainError::PermissionDenied => StatusCode::FORBIDDEN,
            DomainError::UserNotFound | DomainError::ClienteNotFound => StatusCode::NOT_FOUND,
        };

        (status, domain_error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = self.status_and_message();
        (status, Json(json!({ "error": message }))).into_response()
    }
}

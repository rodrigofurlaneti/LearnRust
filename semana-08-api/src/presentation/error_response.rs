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
/// exclusivo da presentation.
pub struct ApiError(ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        Self(error)
    }
}

impl ApiError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match &self.0 {
            ApplicationError::Domain(DomainError::InvalidEmail) => {
                (StatusCode::BAD_REQUEST, DomainError::InvalidEmail.to_string())
            }
            ApplicationError::Domain(DomainError::WeakPassword) => {
                (StatusCode::BAD_REQUEST, DomainError::WeakPassword.to_string())
            }
            ApplicationError::Domain(DomainError::UserAlreadyExists) => (
                StatusCode::CONFLICT,
                DomainError::UserAlreadyExists.to_string(),
            ),
            ApplicationError::Domain(DomainError::InvalidCredentials) => (
                StatusCode::UNAUTHORIZED,
                DomainError::InvalidCredentials.to_string(),
            ),
            ApplicationError::Unexpected(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = self.status_and_message();
        (status, Json(json!({ "error": message }))).into_response()
    }
}

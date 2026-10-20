use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// The one error type every handler returns.
///
/// Add a variant when you need a new kind of failure, then give it a status
/// and a stable `code` below. Clients branch on `code`, so never rename one.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),

    #[error("payment not found")]
    NotFound,

    #[error("{0}")]
    Conflict(String),

    #[error("not implemented yet")]
    NotImplemented,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("Unauthorized request")]
    Unauthorized,

    #[error("{0}")]
    Internal(String),

}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            Self::NotImplemented => (StatusCode::NOT_IMPLEMENTED, "not_implemented"),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED,"unauthorized_request"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR,"internal_error")
        };
        
        // Never send database details to the client. Log them instead.
        let message = match &self {
            Self::Database(e) => {
                tracing::error!(error = %e, "database error");
                "internal server error".to_string()
            },
            Self::Internal(detail) => {
                tracing::error!(%detail, "internal error");
                "internal server error".to_string()
            },
            other => other.to_string(),
        };

        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

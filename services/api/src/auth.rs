use crate::state::AppState;
use axum::{extract::FromRequestParts, http::request::Parts};
use common::errors::ApiError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthMerchant {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthMerchant {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let key = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(ApiError::Unauthorized)?;
        let key_hash = hex::encode(Sha256::digest(key.as_bytes()));

        let merchant_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT merchant_id FROM api_keys WHERE key_hash = $1 AND revoked_At IS NULL",
        )
        .bind(&key_hash)
        .fetch_optional(&state.db)
        .await?;

        merchant_id
            .map(|id| AuthMerchant { id })
            .ok_or(ApiError::Unauthorized)
    }
}

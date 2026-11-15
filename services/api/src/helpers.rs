use axum::http::HeaderMap;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{error::ApiError, payments::dto::CreateRefundRequest};

pub fn hash_refund_request(
    payment_id: Uuid,
    body: &CreateRefundRequest,
) -> Result<String, ApiError> {
    let bytes = serde_json::to_vec(&("refund", payment_id, body))
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

pub fn read_idemp_key(header: &HeaderMap) -> Result<String, ApiError> {
    let value = header
        .get("idemp_key")
        .ok_or_else(|| ApiError::BadRequest("Idempotency header is required!".into()))?;
    let key = value
        .to_str()
        .map_err(|_| ApiError::BadRequest("Idempotency key must be plain text".into()))?
        .trim();
    if key.is_empty() || key.len() > 255 {
        return Err(ApiError::BadRequest(
            "Idempotency key must be 1-255 characters!".into(),
        ));
    }
    Ok(key.to_string())
}

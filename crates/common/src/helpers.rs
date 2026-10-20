use axum::http::HeaderMap;

use crate::errors::ApiError;

pub fn read_idemp_key(header: &HeaderMap)-> Result<String,ApiError> {
    let value = header
        .get("idemp_key")
        .ok_or_else(|| ApiError::BadRequest("Idempotency header is required!".into()))?;
    let key = value
        .to_str()
        .map_err(|_| ApiError::BadRequest("Idempotency key must be plain text".into()))?
        .trim();
    if key.is_empty() || key.len() > 255 {
        return Err(ApiError::BadRequest("Idempotency key must be 1-255 characters!".into()))
    }
    Ok(key.to_string())
}   

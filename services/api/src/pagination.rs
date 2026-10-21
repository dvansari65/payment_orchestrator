use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use common::errors::ApiError;
use uuid::Uuid;

pub fn encode_cursor(id: Uuid) -> String {
    URL_SAFE_NO_PAD.encode(id.as_bytes())
}

pub fn decode_cursor(s: &str) -> Result<Uuid, ApiError> {
    URL_SAFE_NO_PAD
        .decode(s)
        .ok()
        .and_then(|bytes| Uuid::from_slice(&bytes).ok())
        .ok_or_else(|| ApiError::BadRequest("Failed to convert String to UUID:{e}".to_string()))
}

use axum::{Json, extract::State, http::StatusCode};
use persistence::db::ping_db;
use serde_json::{Value, json};

use crate::state::AppState;

pub async fn live() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    match ping_db(&state.db).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "status": "ok" }))),
        Err(e) => {
            tracing::warn!(error = %e, "readiness check failed");
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "status": "unavailable" })))
        }
    }
}

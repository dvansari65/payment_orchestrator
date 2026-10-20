use axum::Router;

use crate::state::AppState;

// Mounted at /api/v1/payments in router.rs, so paths here are relative:
// "/" is /api/v1/payments, "/{id}" is /api/v1/payments/{id}.
pub fn routes() -> Router<AppState> {
    Router::new()
    
}

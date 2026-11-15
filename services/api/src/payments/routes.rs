use axum::{
    Router,
    routing::{get, post},
};

use crate::{payments::handlers, state::AppState};

// Mounted at /api/v1/payments in router.rs, so paths here are relative:
// "/" is /api/v1/payments, "/{id}" is /api/v1/payments/{id}.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/payments",
            get(handlers::get_payment_list).post(handlers::create_payment),
        )
        .route("/payments/{id}", get(handlers::get_payment))
        .route("/payments/{id}/refund", post(handlers::create_refund))
}

use axum::{Router, routing::get};

use super::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/live", get(handlers::live))
        .route("/ready", get(handlers::ready))
}

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{health, payments, state::AppState};

/// The only place that decides URLs. Each feature exposes its routes with
/// paths relative to itself; this file decides where they are mounted.
pub fn create_router(state: AppState) -> Router {
    let api_v1 = Router::new().nest("/payments", payments::routes::routes());

    Router::new()
        .nest("/health", health::routes::routes())
        .nest("/api/v1", api_v1)
        // Layers wrap every route added above them, so they go last.
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

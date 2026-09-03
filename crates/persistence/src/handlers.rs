use axum::{Json};

use crate::models::HealthStatus;


pub async fn health_status()->Json<HealthStatus>{
    Json(
        HealthStatus { status: "Ok".to_string() }
    )
}
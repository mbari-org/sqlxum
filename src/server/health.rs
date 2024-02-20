use axum::{routing, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use utoipa::ToSchema;

pub fn create_router() -> Router {
    Router::new()
        .route("/ping", routing::get(ping))
        .route("/health", routing::get(get_health))
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct Pong {
    /// The time of the pong.
    #[schema(value_type = str)]
    pub pong: DateTime<Utc>,
}

/// Ping the service.
#[utoipa::path(
    get,
    path = "/ping",
    responses(
       (status = 200, description = "Pong", body = Pong)
    )
)]

pub async fn ping() -> Json<Pong> {
    Json(Pong { pong: Utc::now() })
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct HealthStatus {
    pub free_memory: u64,
    pub total_memory: u64,
    pub cpus: usize,
    pub application: String,
    pub version: String,
}

/// Get a basic status of the service.
#[utoipa::path(
    get,
    path = "/health",
    responses(
       (status = 200, description = "Get a status of the service", body = HealthStatus)
    )
)]

pub async fn get_health() -> Json<HealthStatus> {
    Json(get_health_status())
}

pub fn get_health_status() -> HealthStatus {
    let start = Instant::now();
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::new().with_frequency()),
    );
    sys.refresh_memory();
    sys.refresh_cpu();
    let status = HealthStatus {
        free_memory: sys.available_memory(),
        total_memory: sys.total_memory(),
        cpus: sys.cpus().len(),
        application: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let duration = start.elapsed();
    log::info!("get_health_status (took: {duration:?})");
    status
}

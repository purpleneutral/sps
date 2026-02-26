use crate::badge;
use crate::storage::Storage;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use scanner_core::spec::Grade;
use scanner_engine::normalize_domain;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type AppState<S> = Arc<S>;

// ── Request / response types ────────────────────────────────────────

#[derive(Deserialize)]
pub struct ScanRequest {
    pub domain: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub domain: String,
    #[serde(default = "default_interval")]
    pub interval_hours: i32,
}

fn default_interval() -> i32 {
    24
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn json_error(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(ErrorResponse { error: msg.into() })).into_response()
}

// ── Handlers ────────────────────────────────────────────────────────

/// POST /api/scan — trigger a scan and store the result.
pub async fn scan_domain<S: Storage>(
    State(storage): State<AppState<S>>,
    Json(req): Json<ScanRequest>,
) -> Response {
    let domain = normalize_domain(&req.domain);
    if domain.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "Domain is required");
    }

    let result = match scanner_engine::run_scan(&domain).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Scan failed for {domain}: {e:#}");
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Scan failed: {e}"),
            );
        }
    };

    let scan_json = serde_json::to_string(&result).unwrap();
    let grade_str = result.grade.to_string();

    if let Err(e) = storage
        .store_scan(&domain, result.total_score, &grade_str, &scan_json)
        .await
    {
        tracing::error!("Failed to store scan for {domain}: {e:#}");
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to store scan");
    }

    // Update last_scanned_at if registered
    let _ = storage.mark_scanned(&domain).await;

    (StatusCode::OK, Json(result)).into_response()
}

/// GET /api/verify/:domain — get latest scan results.
pub async fn verify_domain<S: Storage>(
    State(storage): State<AppState<S>>,
    Path(domain): Path<String>,
) -> Response {
    let domain = normalize_domain(&domain);

    match storage.get_latest_scan(&domain).await {
        Ok(Some(record)) => {
            // Return the full scan data
            let scan_data: serde_json::Value =
                serde_json::from_str(&record.scan_data).unwrap_or(serde_json::json!(null));
            (StatusCode::OK, Json(scan_data)).into_response()
        }
        Ok(None) => json_error(StatusCode::NOT_FOUND, "No scan found for this domain"),
        Err(e) => {
            tracing::error!("Storage error: {e:#}");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "Storage error")
        }
    }
}

/// GET /api/history/:domain — get scan history.
pub async fn domain_history<S: Storage>(
    State(storage): State<AppState<S>>,
    Path(domain): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Response {
    let domain = normalize_domain(&domain);

    match storage.get_history(&domain, query.limit).await {
        Ok(records) => {
            let summaries: Vec<serde_json::Value> = records
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "domain": r.domain,
                        "score": r.score,
                        "grade": r.grade,
                        "scanned_at": r.scanned_at,
                    })
                })
                .collect();
            (StatusCode::OK, Json(summaries)).into_response()
        }
        Err(e) => {
            tracing::error!("Storage error: {e:#}");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "Storage error")
        }
    }
}

/// GET /api/domains — list all scanned domains.
pub async fn list_domains<S: Storage>(
    State(storage): State<AppState<S>>,
    Query(query): Query<ListQuery>,
) -> Response {
    match storage.list_domains(query.limit, query.offset).await {
        Ok(records) => {
            let summaries: Vec<serde_json::Value> = records
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "domain": r.domain,
                        "score": r.score,
                        "grade": r.grade,
                        "scanned_at": r.scanned_at,
                    })
                })
                .collect();
            (StatusCode::OK, Json(summaries)).into_response()
        }
        Err(e) => {
            tracing::error!("Storage error: {e:#}");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "Storage error")
        }
    }
}

/// POST /api/domains — register a domain for scheduled scanning.
pub async fn register_domain<S: Storage>(
    State(storage): State<AppState<S>>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    let domain = normalize_domain(&req.domain);
    if domain.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "Domain is required");
    }

    match storage.register_domain(&domain, req.interval_hours).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "domain": domain,
                "interval_hours": req.interval_hours,
                "status": "registered"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to register domain: {e:#}");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "Registration failed")
        }
    }
}

/// GET /api/domains/search — search domains by prefix.
pub async fn search_domains<S: Storage>(
    State(storage): State<AppState<S>>,
    Query(query): Query<SearchQuery>,
) -> Response {
    match storage.search_domains(&query.q, query.limit).await {
        Ok(records) => {
            let summaries: Vec<serde_json::Value> = records
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "domain": r.domain,
                        "score": r.score,
                        "grade": r.grade,
                        "scanned_at": r.scanned_at,
                    })
                })
                .collect();
            (StatusCode::OK, Json(summaries)).into_response()
        }
        Err(e) => {
            tracing::error!("Storage error: {e:#}");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "Storage error")
        }
    }
}

/// GET /api/stats — aggregate statistics.
pub async fn get_stats<S: Storage>(
    State(storage): State<AppState<S>>,
) -> Response {
    match storage.get_stats().await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => {
            tracing::error!("Storage error: {e:#}");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "Storage error")
        }
    }
}

/// GET /badge/:domain.svg — dynamic SVG badge.
pub async fn badge_svg<S: Storage>(
    State(storage): State<AppState<S>>,
    Path(filename): Path<String>,
) -> Response {
    let domain = filename.strip_suffix(".svg").unwrap_or(&filename);
    let domain = normalize_domain(domain);

    let svg = match storage.get_latest_scan(&domain).await {
        Ok(Some(record)) => {
            let grade = Grade::from_score(record.score);
            badge::generate_badge(grade, record.score)
        }
        _ => badge::generate_unknown_badge(),
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        svg,
    )
        .into_response()
}

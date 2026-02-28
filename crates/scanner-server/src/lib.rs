pub mod api;
pub mod auth;
pub mod badge;
pub mod rate_limit;
pub mod scheduler;
pub mod storage;

use anyhow::Result;
use axum::http::header::{self, HeaderName};
use axum::http::{HeaderValue, Method as HttpMethod};
use axum::middleware;
use axum::response::Response;
use axum::routing::{get, post};
use axum::Extension;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use storage::AnyStorage;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Start the scanner API server.
pub async fn run_server(host: &str, port: u16, database_url: &str) -> Result<()> {
    let storage = storage::connect(database_url).await?;
    let storage = Arc::new(storage);

    // Start background scheduler
    scheduler::spawn_scheduler(Arc::clone(&storage));

    let app = build_router(storage);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Seglamater Scanner API listening on {addr}");
    tracing::info!("Badge URL: http://{addr}/badge/{{domain}}.svg");
    tracing::info!("API docs: POST /api/scan, GET /api/verify/:domain");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

fn build_cors_layer() -> CorsLayer {
    let origins_str = std::env::var("SPS_CORS_ORIGINS")
        .unwrap_or_else(|_| "https://seglamater.app,https://seglamater.com".to_string());

    let origins: Vec<HeaderValue> = origins_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([HttpMethod::GET, HttpMethod::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-api-key"),
        ])
}

async fn security_headers_middleware(
    req: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    resp
}

fn build_router(storage: Arc<AnyStorage>) -> Router {
    let rate_limit_state = rate_limit::RateLimitState::new();
    rate_limit::spawn_cleanup(&rate_limit_state);

    Router::new()
        // Scan endpoints
        .route("/api/scan", post(api::scan_domain::<AnyStorage>))
        .route(
            "/api/verify/{domain}",
            get(api::verify_domain::<AnyStorage>),
        )
        .route(
            "/api/history/{domain}",
            get(api::domain_history::<AnyStorage>),
        )
        // Domain management
        .route(
            "/api/domains",
            get(api::list_domains::<AnyStorage>).post(api::register_domain::<AnyStorage>),
        )
        .route(
            "/api/domains/search",
            get(api::search_domains::<AnyStorage>),
        )
        // Statistics
        .route("/api/stats", get(api::get_stats::<AnyStorage>))
        // Badge
        .route("/badge/{filename}", get(api::badge_svg::<AnyStorage>))
        // Auth gate on write endpoints (POST/PUT/DELETE only; reads pass through)
        .layer(middleware::from_fn(auth::require_api_key))
        // Per-IP rate limiting (write = 5/min, read = 60/min)
        .layer(middleware::from_fn(rate_limit::rate_limit_middleware))
        .layer(Extension(rate_limit_state))
        // Security response headers
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(CompressionLayer::new())
        .layer(build_cors_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(storage)
}

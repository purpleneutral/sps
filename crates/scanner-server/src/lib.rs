pub mod api;
pub mod badge;
pub mod scheduler;
pub mod storage;

use anyhow::Result;
use axum::routing::{get, post};
use axum::Router;
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

    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(storage: Arc<AnyStorage>) -> Router {
    Router::new()
        // Scan endpoints
        .route("/api/scan", post(api::scan_domain::<AnyStorage>))
        .route("/api/verify/{domain}", get(api::verify_domain::<AnyStorage>))
        .route("/api/history/{domain}", get(api::domain_history::<AnyStorage>))
        // Domain management
        .route(
            "/api/domains",
            get(api::list_domains::<AnyStorage>).post(api::register_domain::<AnyStorage>),
        )
        .route("/api/domains/search", get(api::search_domains::<AnyStorage>))
        // Statistics
        .route("/api/stats", get(api::get_stats::<AnyStorage>))
        // Badge
        .route("/badge/{filename}", get(api::badge_svg::<AnyStorage>))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(storage)
}

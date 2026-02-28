use axum::extract::Extension;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use governor::clock::DefaultClock;
use governor::state::keyed::DashMapStateStore;
use governor::{Quota, RateLimiter};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

type KeyedLimiter = RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock>;

#[derive(Clone)]
pub struct RateLimitState {
    pub(crate) write_limiter: Arc<KeyedLimiter>,
    pub(crate) read_limiter: Arc<KeyedLimiter>,
}

impl RateLimitState {
    pub fn new() -> Self {
        let write_quota = Quota::per_minute(NonZeroU32::new(5).unwrap());
        let read_quota = Quota::per_minute(NonZeroU32::new(60).unwrap());

        Self {
            write_limiter: Arc::new(RateLimiter::dashmap(write_quota)),
            read_limiter: Arc::new(RateLimiter::dashmap(read_quota)),
        }
    }
}

/// Spawn a background task that periodically evicts stale rate limiter entries.
pub fn spawn_cleanup(state: &RateLimitState) {
    let write = Arc::clone(&state.write_limiter);
    let read = Arc::clone(&state.read_limiter);

    tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(10 * 60); // 10 minutes
        loop {
            tokio::time::sleep(interval).await;
            write.retain_recent();
            read.retain_recent();
            tracing::debug!("Rate limiter: evicted stale entries");
        }
    });
}

fn extract_client_ip(req: &axum::extract::Request) -> IpAddr {
    // Try X-Forwarded-For (behind reverse proxy like Traefik/Caddy).
    // Use the last (rightmost) IP — that's the one appended by the trusted proxy.
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            if let Some(last) = val.rsplit(',').next() {
                if let Ok(ip) = last.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Fall back to peer address from ConnectInfo
    req.extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
}

pub async fn rate_limit_middleware(
    Extension(state): Extension<RateLimitState>,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&req);
    let is_write = matches!(*req.method(), Method::POST | Method::PUT | Method::DELETE);

    let limiter = if is_write {
        &state.write_limiter
    } else {
        &state.read_limiter
    };

    match limiter.check_key(&ip) {
        Ok(_) => next.run(req).await,
        Err(_) => {
            let body = serde_json::json!({"error": "Too many requests — please try again later"});
            (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response()
        }
    }
}

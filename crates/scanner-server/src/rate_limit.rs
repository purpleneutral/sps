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
    write_limiter: Arc<KeyedLimiter>,
    read_limiter: Arc<KeyedLimiter>,
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

fn extract_client_ip(req: &axum::extract::Request) -> IpAddr {
    // Try X-Forwarded-For (behind reverse proxy like Traefik/Caddy)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            if let Some(first) = val.split(',').next() {
                if let Ok(ip) = first.trim().parse::<IpAddr>() {
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

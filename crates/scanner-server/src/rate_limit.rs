use axum::Json;
use axum::extract::Extension;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use governor::clock::DefaultClock;
use governor::state::keyed::DashMapStateStore;
use governor::{Quota, RateLimiter};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

type KeyedLimiter = RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock>;

#[derive(Clone)]
pub struct RateLimitState {
    pub(crate) write_limiter_unauth: Arc<KeyedLimiter>,
    pub(crate) write_limiter_auth: Arc<KeyedLimiter>,
    pub(crate) read_limiter: Arc<KeyedLimiter>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimitState {
    pub fn new() -> Self {
        let write_quota_unauth = Quota::per_minute(NonZeroU32::new(2).unwrap());
        let write_quota_auth = Quota::per_minute(NonZeroU32::new(5).unwrap());
        let read_quota = Quota::per_minute(NonZeroU32::new(60).unwrap());

        Self {
            write_limiter_unauth: Arc::new(RateLimiter::dashmap(write_quota_unauth)),
            write_limiter_auth: Arc::new(RateLimiter::dashmap(write_quota_auth)),
            read_limiter: Arc::new(RateLimiter::dashmap(read_quota)),
        }
    }
}

/// Spawn a background task that periodically evicts stale rate limiter entries.
pub fn spawn_cleanup(state: &RateLimitState) {
    let write_unauth = Arc::clone(&state.write_limiter_unauth);
    let write_auth = Arc::clone(&state.write_limiter_auth);
    let read = Arc::clone(&state.read_limiter);

    tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(10 * 60); // 10 minutes
        loop {
            tokio::time::sleep(interval).await;
            write_unauth.retain_recent();
            write_auth.retain_recent();
            read.retain_recent();
            tracing::debug!("Rate limiter: evicted stale entries");
        }
    });
}

/// Check whether the request carries a valid API key.
fn is_authenticated(req: &axum::extract::Request) -> bool {
    let expected = match std::env::var("SPS_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => return false, // no API key configured — nobody is authenticated
    };

    // Check X-API-Key header
    if let Some(val) = req.headers().get("x-api-key")
        && let Ok(provided) = val.to_str()
        && provided == expected
    {
        return true;
    }

    // Check Authorization: Bearer header
    if let Some(val) = req.headers().get("authorization")
        && let Ok(provided) = val.to_str()
        && let Some(token) = provided.strip_prefix("Bearer ")
        && token == expected
    {
        return true;
    }

    false
}

fn extract_client_ip(req: &axum::extract::Request) -> IpAddr {
    // Try X-Forwarded-For (behind reverse proxy like Traefik/Caddy).
    // Use the last (rightmost) IP — that's the one appended by the trusted proxy.
    if let Some(forwarded) = req.headers().get("x-forwarded-for")
        && let Ok(val) = forwarded.to_str()
        && let Some(last) = val.rsplit(',').next()
        && let Ok(ip) = last.trim().parse::<IpAddr>()
    {
        return ip;
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
        if is_authenticated(&req) {
            &state.write_limiter_auth
        } else {
            &state.write_limiter_unauth
        }
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

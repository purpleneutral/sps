use axum::Json;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Constant-time string comparison to prevent timing attacks on API key.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Middleware that requires an API key for write endpoints (POST/PUT/DELETE).
///
/// If `SPS_API_KEY` is unset or empty, all requests pass through (open mode).
/// When set, write requests must include `X-API-Key: <key>` or
/// `Authorization: Bearer <key>`.
pub async fn require_api_key(req: axum::extract::Request, next: Next) -> Response {
    // Only gate write methods
    if !matches!(*req.method(), Method::POST | Method::PUT | Method::DELETE) {
        return next.run(req).await;
    }

    let expected = match std::env::var("SPS_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => return next.run(req).await, // open mode
    };

    // Check X-API-Key header
    if let Some(val) = req.headers().get("x-api-key")
        && let Ok(provided) = val.to_str()
        && constant_time_eq(provided, &expected)
    {
        return next.run(req).await;
    }

    // Check Authorization: Bearer header
    if let Some(val) = req.headers().get("authorization")
        && let Ok(provided) = val.to_str()
        && let Some(token) = provided.strip_prefix("Bearer ")
        && constant_time_eq(token, &expected)
    {
        return next.run(req).await;
    }

    let body = serde_json::json!({"error": "Unauthorized — provide a valid API key"});
    (StatusCode::UNAUTHORIZED, Json(body)).into_response()
}

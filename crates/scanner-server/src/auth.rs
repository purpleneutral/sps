use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;

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
    if let Some(val) = req.headers().get("x-api-key") {
        if let Ok(provided) = val.to_str() {
            if provided == expected {
                return next.run(req).await;
            }
        }
    }

    // Check Authorization: Bearer header
    if let Some(val) = req.headers().get("authorization") {
        if let Ok(provided) = val.to_str() {
            if let Some(token) = provided.strip_prefix("Bearer ") {
                if token == expected {
                    return next.run(req).await;
                }
            }
        }
    }

    let body = serde_json::json!({"error": "Unauthorized — provide a valid API key"});
    (StatusCode::UNAUTHORIZED, Json(body)).into_response()
}

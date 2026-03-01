use serde::{Deserialize, Serialize};

/// All data collected from a headless browser session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserData {
    /// All network requests observed during page load.
    pub network_requests: Vec<NetworkRequest>,
    /// All cookies present after page load (includes JS-set cookies).
    pub cookies: Vec<BrowserCookie>,
    /// The fully rendered HTML after JavaScript execution.
    pub rendered_html: String,
    /// Whether the page loaded successfully.
    pub page_loaded: bool,
    /// Load time in milliseconds.
    pub load_time_ms: u64,
    /// Console errors observed during page load.
    pub console_errors: Vec<String>,
}

/// A network request observed during browser page load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// The full URL requested.
    pub url: String,
    /// The domain extracted from the URL.
    pub domain: String,
    /// HTTP method (GET, POST, etc.).
    pub method: String,
    /// Resource type as reported by CDP (Script, Stylesheet, Image, XHR, Fetch, etc.).
    pub resource_type: String,
    /// HTTP status code of the response (0 if no response received).
    pub status: u16,
    /// MIME type of the response.
    pub mime_type: Option<String>,
    /// The initiator type (script, parser, other).
    pub initiator: String,
    /// Whether the request used HTTPS.
    pub is_https: bool,
}

/// A cookie observed in the browser after page load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserCookie {
    /// Cookie name.
    pub name: String,
    /// Truncated cookie value for privacy (first 8 chars + "...").
    pub value_preview: String,
    /// Domain the cookie is set for.
    pub domain: String,
    /// Path attribute.
    pub path: String,
    /// Whether Secure flag is set.
    pub secure: bool,
    /// Whether HttpOnly flag is set.
    pub http_only: bool,
    /// SameSite value if present.
    pub same_site: Option<String>,
    /// Expiration as seconds from now (None = session cookie).
    pub expires_seconds: Option<i64>,
}

mod recommendations;
pub mod validate;

use anyhow::{bail, Context, Result};
use hickory_resolver::TokioResolver;
use scanner_core::browser_types::BrowserData;
use scanner_core::check::CategoryResult;
use scanner_core::score::ScanResult;
use scanner_core::ssrf::is_private_ip;
use std::net::{IpAddr, SocketAddr};
#[cfg(feature = "browser")]
use std::sync::LazyLock;
#[cfg(feature = "browser")]
use tokio::sync::Semaphore;

pub use recommendations::generate_recommendations;
pub use validate::validate_domain;

/// Maximum response body size (10 MB).
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Maximum concurrent browser sessions.
#[cfg(feature = "browser")]
static BROWSER_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| {
    let max = std::env::var("SPS_MAX_BROWSER_SESSIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    Semaphore::new(max)
});

/// Attempt to fetch browser data. Returns None if the browser feature is
/// disabled or if the browser fetch fails for any reason.
async fn fetch_browser_data(domain: &str) -> Option<BrowserData> {
    #[cfg(feature = "browser")]
    {
        let permit = match BROWSER_SEMAPHORE.try_acquire() {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!(
                    "Browser session limit reached, skipping browser fetch for {domain}"
                );
                return None;
            }
        };

        let result = scanner_browser::fetch_with_browser(domain).await;
        drop(permit);

        match result {
            Ok(data) => {
                tracing::info!(
                    "Browser fetch complete: {} requests, {} cookies, {}ms",
                    data.network_requests.len(),
                    data.cookies.len(),
                    data.load_time_ms,
                );
                Some(data)
            }
            Err(e) => {
                tracing::warn!("Browser fetch failed, using static analysis only: {e}");
                None
            }
        }
    }
    #[cfg(not(feature = "browser"))]
    {
        let _ = domain;
        None
    }
}

/// Run a full scan against a domain and return the complete result.
pub async fn run_scan(domain: &str) -> Result<ScanResult> {
    validate::validate_domain(domain).await?;

    let (headers, html, set_cookie_headers) = fetch_page(domain).await?;

    // Optionally fetch browser data for enhanced detection
    let browser_data = fetch_browser_data(domain).await;

    let page_url = format!("https://{domain}/");

    // Run checks in parallel where possible
    let (transport, dns) = tokio::join!(
        scanner_transport::check_transport(domain),
        scanner_dns::check_dns(domain),
    );

    // These depend on the fetched page data
    let security_headers = scanner_headers::check_headers(&headers);
    let tracking =
        scanner_tracking::check_tracking(domain, &html, &page_url, browser_data.as_ref());
    let cookies =
        scanner_cookies::check_cookies(&set_cookie_headers, domain, browser_data.as_ref());

    let best_practices =
        scanner_bestpractices::check_best_practices(domain, &html, browser_data.as_ref()).await;

    let categories: Vec<CategoryResult> = vec![
        transport,
        security_headers,
        tracking,
        cookies,
        dns,
        best_practices,
    ];

    let recs = recommendations::generate(&categories);

    Ok(ScanResult::from_categories(
        domain.to_string(),
        categories,
        recs,
    ))
}

/// Fetch the page and extract headers, HTML body, and Set-Cookie headers.
pub async fn fetch_page(
    domain: &str,
) -> Result<(reqwest::header::HeaderMap, String, Vec<String>)> {
    // Resolve DNS and pin to prevent rebinding attacks
    let resolver = TokioResolver::builder_tokio()
        .map_err(|_| anyhow::anyhow!("Failed to create DNS resolver"))?
        .build();

    let ips: Vec<IpAddr> = resolver
        .lookup_ip(domain)
        .await
        .context(format!("DNS resolution failed for {domain}"))?
        .iter()
        .collect();

    let pin_ip = ips
        .iter()
        .find(|ip| !is_private_ip(ip))
        .ok_or_else(|| anyhow::anyhow!("Domain resolves to private IPs only"))?;

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .resolve(domain, SocketAddr::new(*pin_ip, 443))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 10 {
                attempt.stop()
            } else if let Some(host) = attempt.url().host_str() {
                if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                    if scanner_core::ssrf::is_private_ip(&ip) {
                        attempt.stop()
                    } else {
                        attempt.follow()
                    }
                } else {
                    attempt.follow()
                }
            } else {
                attempt.follow()
            }
        }))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(
            "Mozilla/5.0 (compatible; SeglamaterScan/0.1; +https://seglamater.app/privacy)",
        )
        .build()
        .context("Failed to build HTTP client")?;

    let url = format!("https://{domain}");
    let resp = client
        .get(&url)
        .send()
        .await
        .context(format!("Failed to connect to {domain}"))?;

    let headers = resp.headers().clone();

    let set_cookie_headers: Vec<String> = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .collect();

    // Enforce body size limit
    if let Some(len) = resp.content_length() {
        if len > MAX_BODY_SIZE as u64 {
            bail!("Response body too large");
        }
    }

    let bytes = resp
        .bytes()
        .await
        .context("Failed to read response body")?;

    if bytes.len() > MAX_BODY_SIZE {
        bail!("Response body too large");
    }

    let html = String::from_utf8_lossy(&bytes).into_owned();

    Ok((headers, html, set_cookie_headers))
}

/// Normalize domain input: strip protocol, trailing slashes, paths, ports.
pub fn normalize_domain(input: &str) -> String {
    let mut domain = input.trim().to_string();

    // Strip protocol
    if let Some(rest) = domain.strip_prefix("https://") {
        domain = rest.to_string();
    } else if let Some(rest) = domain.strip_prefix("http://") {
        domain = rest.to_string();
    }

    // Strip path and trailing slash
    if let Some(idx) = domain.find('/') {
        domain = domain[..idx].to_string();
    }

    // Strip port
    if let Some(idx) = domain.find(':') {
        domain = domain[..idx].to_string();
    }

    domain.to_lowercase()
}

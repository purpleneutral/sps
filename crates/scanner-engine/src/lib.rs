mod recommendations;
pub mod validate;

use anyhow::{Context, Result};
use scanner_core::check::CategoryResult;
use scanner_core::score::ScanResult;

pub use recommendations::generate_recommendations;
pub use validate::validate_domain;

/// Run a full scan against a domain and return the complete result.
pub async fn run_scan(domain: &str) -> Result<ScanResult> {
    validate::validate_domain(domain).await?;

    let (headers, html, set_cookie_headers) = fetch_page(domain).await?;

    let page_url = format!("https://{domain}/");

    // Run checks in parallel where possible
    let (transport, dns) = tokio::join!(
        scanner_transport::check_transport(domain),
        scanner_dns::check_dns(domain),
    );

    // These depend on the fetched page data
    let security_headers = scanner_headers::check_headers(&headers);
    let tracking = scanner_tracking::check_tracking(domain, &html, &page_url);
    let cookies = scanner_cookies::check_cookies(&set_cookie_headers, domain);

    let best_practices = scanner_bestpractices::check_best_practices(domain, &html).await;

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
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 10 {
                attempt.stop()
            } else if let Some(host) = attempt.url().host_str() {
                if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                    if crate::validate::is_private_ip(&ip) {
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
        .user_agent("Mozilla/5.0 (compatible; SeglamaterScan/0.1; +https://seglamater.com/scan)")
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

    let html = resp
        .text()
        .await
        .context("Failed to read response body")?;

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

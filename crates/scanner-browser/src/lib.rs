mod collect;
mod launch;
mod navigate;
mod wait;

use anyhow::Result;
use scanner_core::browser_types::BrowserData;

/// Default timeout for the entire browser fetch operation.
const BROWSER_TIMEOUT_SECS: u64 = 45;

/// Fetch a page using headless Chromium and collect runtime data.
///
/// Launches a fresh browser instance, navigates to `https://{domain}/`,
/// waits for network idle, collects all network requests, cookies, and
/// rendered HTML, then closes the browser.
pub async fn fetch_with_browser(domain: &str) -> Result<BrowserData> {
    let deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(BROWSER_TIMEOUT_SECS);

    match tokio::time::timeout_at(deadline, fetch_inner(domain)).await {
        Ok(inner) => inner,
        Err(_) => anyhow::bail!("Browser fetch timed out after {BROWSER_TIMEOUT_SECS}s"),
    }
}

async fn fetch_inner(domain: &str) -> Result<BrowserData> {
    // 1. SSRF pre-check: validate domain resolves to public IPs
    let resolved_ip = navigate::validate_target(domain).await?;

    // 2. Launch browser with DNS pinning for the target domain
    let (mut browser, handler) = launch::launch_browser(domain, &resolved_ip).await?;
    let handler_handle = tokio::spawn(async move {
        futures::pin_mut!(handler);
        loop {
            if futures::StreamExt::next(&mut handler).await.is_none() {
                break;
            }
        }
    });

    // 3. Create page and set up network interception
    let page = browser.new_page("about:blank").await?;
    let network_log = navigate::setup_network_interception(&page).await?;

    // 4. Navigate
    let start = std::time::Instant::now();
    let url = format!("https://{domain}/");
    page.goto(&url).await?;

    // 5. Wait for network idle
    wait::wait_for_idle(&page).await;

    let load_time_ms = start.elapsed().as_millis() as u64;

    // 6. Collect data
    let rendered_html = collect::get_rendered_html(&page).await.unwrap_or_default();
    let cookies = collect::get_cookies(&page).await.unwrap_or_default();
    let network_requests = network_log.lock().await.clone();

    // 7. Cleanup
    drop(page);
    let _ = browser.close().await;
    handler_handle.abort();

    Ok(BrowserData {
        network_requests,
        cookies,
        rendered_html,
        page_loaded: true,
        load_time_ms,
        console_errors: Vec::new(),
    })
}

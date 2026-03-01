use chromiumoxide::Page;
use std::time::Duration;

/// Wait for the page to reach network idle.
///
/// Uses a PerformanceObserver to detect when no new resources are loading,
/// bounded by a maximum timeout. Falls back gracefully on errors.
pub async fn wait_for_idle(page: &Page) {
    let idle_js = r#"
        new Promise((resolve) => {
            let timer = null;
            const observer = new PerformanceObserver(() => {
                clearTimeout(timer);
                timer = setTimeout(resolve, 500);
            });
            observer.observe({ type: 'resource', buffered: false });
            timer = setTimeout(resolve, 500);
        })
    "#;

    let timeout = Duration::from_secs(10);
    match tokio::time::timeout(timeout, page.evaluate(idle_js)).await {
        Ok(Ok(_)) => {
            tracing::debug!("Page reached network idle");
        }
        Ok(Err(e)) => {
            tracing::warn!("Network idle detection failed, continuing: {e}");
        }
        Err(_) => {
            tracing::warn!("Network idle wait timed out after {timeout:?}, continuing");
        }
    }

    // Small buffer for late-firing scripts
    tokio::time::sleep(Duration::from_millis(500)).await;
}

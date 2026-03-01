use anyhow::Result;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::Handler;

/// Chrome flags for a secure, sandboxed headless session.
const CHROME_ARGS: &[&str] = &[
    "--headless=new",
    "--disable-gpu",
    "--no-sandbox",
    "--disable-dev-shm-usage",
    "--disable-background-networking",
    "--disable-default-apps",
    "--disable-extensions",
    "--disable-sync",
    "--disable-translate",
    "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows",
    "--disable-renderer-backgrounding",
    "--metrics-recording-only",
    "--no-first-run",
    "--mute-audio",
    "--hide-scrollbars",
    "--disable-file-system",
    "--js-flags=--max-old-space-size=256",
    "--incognito",
];

/// Launch a headless Chromium instance with DNS pinning for the target domain.
pub async fn launch_browser(domain: &str, resolved_ip: &str) -> Result<(Browser, Handler)> {
    let chrome_bin = std::env::var("CHROME_BIN").unwrap_or_else(|_| "chromium".to_string());
    let pin_rule = format!("--host-resolver-rules=MAP {domain} {resolved_ip}");

    let mut args: Vec<String> = CHROME_ARGS.iter().map(|s| s.to_string()).collect();
    args.push(pin_rule);

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let config = BrowserConfig::builder()
        .chrome_executable(chrome_bin)
        .args(arg_refs)
        .request_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build browser config: {e}"))?;

    let (browser, handler) = Browser::launch(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to launch browser: {e}"))?;

    Ok((browser, handler))
}

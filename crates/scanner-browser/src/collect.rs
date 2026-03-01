use anyhow::Result;
use chromiumoxide::Page;
use scanner_core::browser_types::BrowserCookie;

/// Get the fully rendered HTML from the page after JS execution.
pub async fn get_rendered_html(page: &Page) -> Result<String> {
    let html = page
        .evaluate("document.documentElement.outerHTML")
        .await?
        .into_value::<String>()?;

    Ok(html)
}

/// Get all cookies from the browser, including JS-set cookies.
pub async fn get_cookies(page: &Page) -> Result<Vec<BrowserCookie>> {
    let cdp_cookies = page.get_cookies().await?;
    let now = chrono::Utc::now().timestamp() as f64;

    let cookies = cdp_cookies
        .into_iter()
        .map(|c| {
            let expires_seconds = if c.expires > 0.0 {
                Some((c.expires - now) as i64)
            } else {
                None
            };

            let same_site = c.same_site.map(|ss| format!("{ss:?}").to_lowercase());

            let value_preview = if c.value.len() > 8 {
                format!("{}...", &c.value[..8])
            } else {
                c.value.clone()
            };

            BrowserCookie {
                name: c.name,
                value_preview,
                domain: c.domain,
                path: c.path,
                secure: c.secure,
                http_only: c.http_only,
                same_site,
                expires_seconds,
            }
        })
        .collect();

    Ok(cookies)
}

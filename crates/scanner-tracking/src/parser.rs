use scraper::{Html, Selector};
use url::Url;

/// An external resource found in the page HTML or observed via browser.
#[derive(Debug, Clone)]
pub struct ExternalResource {
    /// The full URL of the resource.
    pub url: String,
    /// The domain of the resource.
    pub domain: String,
    /// The type of HTML element or source (e.g., "script", "link", "img", "dynamic").
    pub element: String,
    /// Whether the URL uses HTTPS.
    pub is_https: bool,
}

/// Extract all external resource URLs from HTML.
pub fn extract_external_resources(
    html: &str,
    page_url: &str,
    _first_party_domain: &str,
) -> Vec<ExternalResource> {
    let document = Html::parse_document(html);
    let base_url = Url::parse(page_url).ok();
    let mut resources = Vec::new();

    // <script src="...">
    if let Ok(sel) = Selector::parse("script[src]") {
        for el in document.select(&sel) {
            if let Some(src) = el.value().attr("src") {
                if let Some(r) = resolve_resource(src, base_url.as_ref(), "script") {
                    resources.push(r);
                }
            }
        }
    }

    // <link href="..."> (stylesheets, fonts, etc.)
    if let Ok(sel) = Selector::parse("link[href]") {
        for el in document.select(&sel) {
            if let Some(href) = el.value().attr("href") {
                if let Some(r) = resolve_resource(href, base_url.as_ref(), "link") {
                    resources.push(r);
                }
            }
        }
    }

    // <img src="...">
    if let Ok(sel) = Selector::parse("img[src]") {
        for el in document.select(&sel) {
            if let Some(src) = el.value().attr("src") {
                if let Some(r) = resolve_resource(src, base_url.as_ref(), "img") {
                    resources.push(r);
                }
            }
        }
    }

    // <iframe src="...">
    if let Ok(sel) = Selector::parse("iframe[src]") {
        for el in document.select(&sel) {
            if let Some(src) = el.value().attr("src") {
                if let Some(r) = resolve_resource(src, base_url.as_ref(), "iframe") {
                    resources.push(r);
                }
            }
        }
    }

    resources
}

fn resolve_resource(
    raw_url: &str,
    base: Option<&Url>,
    element: &str,
) -> Option<ExternalResource> {
    let url = if raw_url.starts_with("//") {
        Url::parse(&format!("https:{raw_url}")).ok()
    } else if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
        Url::parse(raw_url).ok()
    } else if let Some(base) = base {
        base.join(raw_url).ok()
    } else {
        None
    };

    let url = url?;
    let domain = url.host_str()?.to_string();
    let is_https = url.scheme() == "https";

    Some(ExternalResource {
        url: url.to_string(),
        domain,
        element: element.to_string(),
        is_https,
    })
}

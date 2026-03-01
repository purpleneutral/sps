pub mod blocklists;
pub mod fingerprint;
pub mod parser;
pub mod resources;

use scanner_core::browser_types::BrowserData;
use scanner_core::check::{CategoryResult, CheckResult};
use scanner_core::spec::Category;

const CAT: Category = Category::TrackingThirdParties;

/// Run all tracking and third-party checks.
///
/// `domain` is the first-party domain being scanned.
/// `html` is the HTML body of the page.
/// `page_url` is the full URL that was fetched.
/// `browser_data` contains runtime browser observations when available.
pub fn check_tracking(
    domain: &str,
    html: &str,
    page_url: &str,
    browser_data: Option<&BrowserData>,
) -> CategoryResult {
    let mut external_resources = parser::extract_external_resources(html, page_url, domain);

    // Merge browser-observed network requests as additional resources
    if let Some(bd) = browser_data {
        for req in &bd.network_requests {
            // Skip first-party requests and non-resource types
            if req.domain.is_empty() || req.domain == domain {
                continue;
            }
            // Only include resource types that indicate external loading
            let element = match req.resource_type.as_str() {
                "Script" => "script",
                "Stylesheet" => "link",
                "Image" => "img",
                _ => "dynamic",
            };
            // Deduplicate by URL
            if !external_resources.iter().any(|er| er.url == req.url) {
                external_resources.push(parser::ExternalResource {
                    url: req.url.clone(),
                    domain: req.domain.clone(),
                    element: element.to_string(),
                    is_https: req.is_https,
                });
            }
        }
    }

    let analytics = resources::find_analytics(&external_resources);
    let trackers = resources::find_trackers(&external_resources);
    let third_party_cdns = resources::find_third_party_cdns(&external_resources, domain);
    let has_mixed_content = resources::has_mixed_content(&external_resources);

    // Run fingerprint detection on both static and rendered HTML
    let mut fingerprinting = fingerprint::detect_fingerprinting(html);
    if let Some(bd) = browser_data {
        for fp in fingerprint::detect_fingerprinting(&bd.rendered_html) {
            if !fingerprinting.contains(&fp) {
                fingerprinting.push(fp);
            }
        }
    }

    let mut checks = Vec::new();

    // No third-party analytics (10 pts)
    if analytics.is_empty() {
        checks.push(CheckResult::pass(
            CAT,
            "no_analytics",
            "No third-party analytics scripts",
            10,
            None,
        ));
    } else {
        checks.push(CheckResult::fail(
            CAT,
            "no_analytics",
            "No third-party analytics scripts",
            10,
            Some(format!(
                "{} analytics script(s) detected: {}",
                analytics.len(),
                analytics.join(", ")
            )),
        ));
    }

    // No third-party advertising/tracking scripts (10 pts)
    if trackers.is_empty() {
        checks.push(CheckResult::pass(
            CAT,
            "no_trackers",
            "No third-party advertising/tracking scripts",
            10,
            None,
        ));
    } else {
        checks.push(CheckResult::fail(
            CAT,
            "no_trackers",
            "No third-party advertising/tracking scripts",
            10,
            Some(format!(
                "{} tracking script(s) detected: {}",
                trackers.len(),
                trackers.join(", ")
            )),
        ));
    }

    // No fingerprinting patterns (5 pts)
    if fingerprinting.is_empty() {
        checks.push(CheckResult::pass(
            CAT,
            "no_fingerprinting",
            "No known fingerprinting patterns detected",
            5,
            None,
        ));
    } else {
        checks.push(CheckResult::fail(
            CAT,
            "no_fingerprinting",
            "No known fingerprinting patterns detected",
            5,
            Some(format!("Detected: {}", fingerprinting.join(", "))),
        ));
    }

    // No third-party CDNs (3 pts)
    if third_party_cdns.is_empty() {
        checks.push(CheckResult::pass(
            CAT,
            "no_third_party_cdns",
            "No third-party fonts/CDNs (bonus)",
            3,
            Some("All resources served from first-party domain".into()),
        ));
    } else {
        checks.push(CheckResult::fail(
            CAT,
            "no_third_party_cdns",
            "No third-party fonts/CDNs (bonus)",
            3,
            Some(format!(
                "{} third-party CDN resource(s): {}",
                third_party_cdns.len(),
                third_party_cdns.join(", ")
            )),
        ));
    }

    // All external resources over HTTPS (2 pts)
    if !has_mixed_content {
        checks.push(CheckResult::pass(
            CAT,
            "all_https",
            "All external resources loaded over HTTPS",
            2,
            None,
        ));
    } else {
        checks.push(CheckResult::fail(
            CAT,
            "all_https",
            "All external resources loaded over HTTPS",
            2,
            Some("Mixed content detected (HTTP resources on HTTPS page)".into()),
        ));
    }

    CategoryResult::new(CAT, checks)
}

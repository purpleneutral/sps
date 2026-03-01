pub mod analysis;

use analysis::CookieInfo;
use scanner_core::browser_types::BrowserData;
use scanner_core::check::{CategoryResult, CheckResult};
use scanner_core::spec::Category;

const CAT: Category = Category::CookieBehavior;
const ONE_YEAR_SECS: i64 = 365 * 24 * 60 * 60;

/// Run all cookie behavior checks.
///
/// `set_cookie_headers` are the raw Set-Cookie header values from the response.
/// `domain` is the first-party domain being scanned.
/// `browser_data` contains runtime browser observations when available.
pub fn check_cookies(
    set_cookie_headers: &[String],
    domain: &str,
    browser_data: Option<&BrowserData>,
) -> CategoryResult {
    let mut cookies: Vec<CookieInfo> = set_cookie_headers
        .iter()
        .map(|raw| analysis::parse_set_cookie(raw, domain))
        .collect();

    // Merge browser-observed cookies (includes JS-set cookies)
    if let Some(bd) = browser_data {
        for bc in &bd.cookies {
            if !cookies.iter().any(|c| c.name == bc.name) {
                cookies.push(CookieInfo {
                    name: bc.name.clone(),
                    secure: bc.secure,
                    http_only: bc.http_only,
                    same_site: bc.same_site.clone(),
                    max_age_seconds: bc.expires_seconds,
                    domain: Some(bc.domain.clone()),
                    is_third_party: !analysis::is_same_domain(&bc.domain, domain),
                });
            }
        }
    }

    let mut checks = Vec::new();

    // No third-party cookies (5 pts)
    let third_party: Vec<&CookieInfo> = cookies.iter().filter(|c| c.is_third_party).collect();
    if third_party.is_empty() {
        checks.push(CheckResult::pass(
            CAT,
            "no_third_party_cookies",
            "No third-party cookies",
            5,
            if cookies.is_empty() {
                Some("No cookies set".into())
            } else {
                None
            },
        ));
    } else {
        checks.push(CheckResult::fail(
            CAT,
            "no_third_party_cookies",
            "No third-party cookies",
            5,
            Some(format!(
                "{} third-party cookie(s) detected",
                third_party.len()
            )),
        ));
    }

    if cookies.is_empty() {
        // No cookies at all — pass everything
        checks.push(CheckResult::pass(
            CAT,
            "cookies_secure",
            "All cookies have Secure flag",
            3,
            Some("No cookies set".into()),
        ));
        checks.push(CheckResult::pass(
            CAT,
            "cookies_httponly",
            "All cookies have HttpOnly flag",
            3,
            Some("No cookies set".into()),
        ));
        checks.push(CheckResult::pass(
            CAT,
            "cookies_samesite",
            "All cookies have SameSite attribute",
            2,
            Some("No cookies set".into()),
        ));
        checks.push(CheckResult::pass(
            CAT,
            "cookies_expiration",
            "Reasonable cookie expiration",
            2,
            Some("No cookies set".into()),
        ));
    } else {
        // All cookies have Secure flag (3 pts)
        let missing_secure: Vec<&str> = cookies
            .iter()
            .filter(|c| !c.secure)
            .map(|c| c.name.as_str())
            .collect();
        if missing_secure.is_empty() {
            checks.push(CheckResult::pass(
                CAT,
                "cookies_secure",
                "All cookies have Secure flag",
                3,
                None,
            ));
        } else {
            checks.push(CheckResult::fail(
                CAT,
                "cookies_secure",
                "All cookies have Secure flag",
                3,
                Some(format!(
                    "{} cookie(s) missing Secure: {}",
                    missing_secure.len(),
                    missing_secure.join(", ")
                )),
            ));
        }

        // All cookies have HttpOnly flag (3 pts)
        let missing_httponly: Vec<&str> = cookies
            .iter()
            .filter(|c| !c.http_only)
            .map(|c| c.name.as_str())
            .collect();
        if missing_httponly.is_empty() {
            checks.push(CheckResult::pass(
                CAT,
                "cookies_httponly",
                "All cookies have HttpOnly flag",
                3,
                None,
            ));
        } else {
            checks.push(CheckResult::fail(
                CAT,
                "cookies_httponly",
                "All cookies have HttpOnly flag",
                3,
                Some(format!(
                    "{} cookie(s) missing HttpOnly: {}",
                    missing_httponly.len(),
                    missing_httponly.join(", ")
                )),
            ));
        }

        // All cookies have SameSite attribute (2 pts)
        let missing_samesite: Vec<&str> = cookies
            .iter()
            .filter(|c| c.same_site.is_none())
            .map(|c| c.name.as_str())
            .collect();
        if missing_samesite.is_empty() {
            checks.push(CheckResult::pass(
                CAT,
                "cookies_samesite",
                "All cookies have SameSite attribute",
                2,
                None,
            ));
        } else {
            checks.push(CheckResult::fail(
                CAT,
                "cookies_samesite",
                "All cookies have SameSite attribute",
                2,
                Some(format!(
                    "{} cookie(s) missing SameSite: {}",
                    missing_samesite.len(),
                    missing_samesite.join(", ")
                )),
            ));
        }

        // Reasonable cookie expiration (2 pts)
        let long_lived: Vec<&str> = cookies
            .iter()
            .filter(|c| c.max_age_seconds.is_some_and(|age| age > ONE_YEAR_SECS))
            .map(|c| c.name.as_str())
            .collect();
        if long_lived.is_empty() {
            checks.push(CheckResult::pass(
                CAT,
                "cookies_expiration",
                "Reasonable cookie expiration",
                2,
                None,
            ));
        } else {
            checks.push(CheckResult::fail(
                CAT,
                "cookies_expiration",
                "Reasonable cookie expiration",
                2,
                Some(format!(
                    "{} cookie(s) with expiration > 1 year: {}",
                    long_lived.len(),
                    long_lived.join(", ")
                )),
            ));
        }
    }

    CategoryResult::new(CAT, checks)
}

pub mod csp;
pub mod misc;
pub mod permissions;
pub mod referrer;

use reqwest::header::HeaderMap;
use scanner_core::check::CategoryResult;
use scanner_core::spec::Category;

const CAT: Category = Category::SecurityHeaders;

/// Run all security header checks. Takes pre-fetched headers to avoid redundant requests.
pub fn check_headers(headers: &HeaderMap) -> CategoryResult {
    let mut checks = Vec::new();

    // CSP checks
    checks.extend(csp::check_csp(headers));

    // Referrer-Policy
    checks.push(referrer::check_referrer_policy(headers));

    // Permissions-Policy
    checks.push(permissions::check_permissions_policy(headers));

    // Misc headers
    checks.extend(misc::check_misc_headers(headers));

    CategoryResult::new(CAT, checks)
}

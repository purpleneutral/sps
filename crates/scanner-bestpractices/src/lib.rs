pub mod accessibility;
pub mod privacy_json;
pub mod security_txt;

use scanner_core::browser_types::BrowserData;
use scanner_core::check::CategoryResult;
use scanner_core::spec::Category;

const CAT: Category = Category::BestPractices;

/// Run all best practices checks.
pub async fn check_best_practices(
    domain: &str,
    html: &str,
    browser_data: Option<&BrowserData>,
) -> CategoryResult {
    let mut checks = Vec::new();

    checks.push(security_txt::check_security_txt(domain).await);
    checks.push(privacy_json::check_privacy_json(domain).await);
    checks.push(accessibility::check_js_free(html, browser_data));

    CategoryResult::new(CAT, checks)
}

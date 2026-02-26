pub mod hsts;
pub mod tls;

use scanner_core::check::CategoryResult;
use scanner_core::spec::Category;

const CAT: Category = Category::TransportSecurity;

/// Run all transport security checks against a domain.
pub async fn check_transport(domain: &str) -> CategoryResult {
    let mut checks = Vec::new();

    // TLS checks
    let (supports_tls13, legacy_disabled) = tls::check_tls(domain).await;
    checks.push(supports_tls13);
    checks.push(legacy_disabled);

    // HSTS checks — need to fetch headers
    let hsts_checks = hsts::check_hsts(domain).await;
    checks.extend(hsts_checks);

    CategoryResult::new(CAT, checks)
}

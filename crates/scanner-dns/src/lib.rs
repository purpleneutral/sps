pub mod caa;
pub mod dkim;
pub mod dmarc;
pub mod dnssec;
pub mod spf;

use hickory_resolver::TokioResolver;
use scanner_core::check::CategoryResult;
use scanner_core::spec::Category;

const CAT: Category = Category::EmailDnsSecurity;

/// Run all DNS and email security checks.
pub async fn check_dns(domain: &str) -> CategoryResult {
    let resolver = TokioResolver::builder_tokio()
        .expect("Failed to create DNS resolver")
        .build();

    let mut checks = Vec::new();

    checks.push(spf::check_spf(domain, &resolver).await);
    checks.push(dkim::check_dkim(domain, &resolver).await);
    checks.push(dmarc::check_dmarc(domain, &resolver).await);
    checks.push(dnssec::check_dnssec(domain, &resolver).await);
    checks.push(caa::check_caa(domain, &resolver).await);

    CategoryResult::new(CAT, checks)
}

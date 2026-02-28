pub mod caa;
pub mod dkim;
pub mod dmarc;
pub mod dnssec;
pub mod spf;

use hickory_resolver::TokioResolver;
use scanner_core::check::{CategoryResult, CheckResult};
use scanner_core::spec::Category;

const CAT: Category = Category::EmailDnsSecurity;

/// Run all DNS and email security checks.
pub async fn check_dns(domain: &str) -> CategoryResult {
    let resolver = match TokioResolver::builder_tokio() {
        Ok(builder) => builder.build(),
        Err(e) => {
            tracing::error!("Failed to create DNS resolver: {e}");
            return CategoryResult::new(
                CAT,
                vec![CheckResult::fail(
                    CAT,
                    "dns_resolver",
                    "DNS resolver available",
                    0,
                    Some("DNS resolver initialization failed".into()),
                )],
            );
        }
    };

    let mut checks = Vec::new();

    checks.push(spf::check_spf(domain, &resolver).await);
    checks.push(dkim::check_dkim(domain, &resolver).await);
    checks.push(dmarc::check_dmarc(domain, &resolver).await);
    checks.push(dnssec::check_dnssec(domain, &resolver).await);
    checks.push(caa::check_caa(domain, &resolver).await);

    CategoryResult::new(CAT, checks)
}

use hickory_resolver::TokioResolver;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::EmailDnsSecurity;

/// Check if DNSSEC is enabled for the domain.
///
/// We check by doing a DNSKEY lookup. If DNSKEY records exist, DNSSEC is deployed.
/// Note: full DNSSEC validation requires checking the chain of trust, but the presence
/// of DNSKEY records is a reasonable indicator.
pub async fn check_dnssec(domain: &str, resolver: &TokioResolver) -> CheckResult {
    // Try to look up DNSKEY records. If they exist, DNSSEC is likely deployed.
    // hickory-resolver doesn't directly expose DNSKEY/DS lookups through the high-level API,
    // so we check if the domain responds with authenticated data by looking for RRSIG.
    // As a practical approach, we check for a DNSKEY record via TXT lookup as a proxy.

    // A more thorough approach would use hickory-client to make raw DNS queries.
    // For now, we try a simple heuristic: query for the domain and check if
    // we get any response that indicates DNSSEC signing.

    match resolver.lookup(domain, hickory_proto::rr::RecordType::DNSKEY).await {
        Ok(response) => {
            if response.iter().next().is_some() {
                CheckResult::pass(
                    CAT,
                    "dnssec_enabled",
                    "DNSSEC enabled",
                    1,
                    Some("DNSKEY records found".into()),
                )
            } else {
                CheckResult::fail(
                    CAT,
                    "dnssec_enabled",
                    "DNSSEC enabled",
                    1,
                    Some("No DNSKEY records found".into()),
                )
            }
        }
        Err(_) => CheckResult::fail(
            CAT,
            "dnssec_enabled",
            "DNSSEC enabled",
            1,
            Some("No DNSKEY records found".into()),
        ),
    }
}

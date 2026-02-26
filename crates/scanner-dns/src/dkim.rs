use hickory_resolver::TokioResolver;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::EmailDnsSecurity;

/// Common DKIM selector names to probe.
const COMMON_SELECTORS: &[&str] = &[
    "default",
    "google",
    "selector1",
    "selector2",
    "k1",
    "k2",
    "dkim",
    "mail",
    "s1",
    "s2",
    "smtp",
    "mandrill",
    "everlytickey1",
    "everlytickey2",
    "mxvault",
];

/// Check if DKIM records are discoverable.
/// DKIM uses selector-based DNS records, so we probe common selectors.
pub async fn check_dkim(domain: &str, resolver: &TokioResolver) -> CheckResult {
    for selector in COMMON_SELECTORS {
        let query = format!("{selector}._domainkey.{domain}");
        if let Ok(response) = resolver.txt_lookup(query.as_str()).await {
            for record in response.iter() {
                let txt = record.to_string();
                if txt.contains("v=DKIM1") || txt.contains("p=") {
                    return CheckResult::pass(
                        CAT,
                        "dkim_present",
                        "DKIM record discoverable",
                        2,
                        Some(format!("Found via selector \"{selector}\"")),
                    );
                }
            }
        }
    }

    CheckResult::fail(
        CAT,
        "dkim_present",
        "DKIM record discoverable",
        2,
        Some("No DKIM record found via common selectors".into()),
    )
}

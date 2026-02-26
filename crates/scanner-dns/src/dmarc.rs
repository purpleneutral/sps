use hickory_resolver::TokioResolver;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::EmailDnsSecurity;

/// Check DMARC policy: set to quarantine or reject.
pub async fn check_dmarc(domain: &str, resolver: &TokioResolver) -> CheckResult {
    let dmarc_domain = format!("_dmarc.{domain}");

    match resolver.txt_lookup(dmarc_domain.as_str()).await {
        Ok(response) => {
            for record in response.iter() {
                let txt = record.to_string();
                if txt.starts_with("v=DMARC1") {
                    if let Some(policy) = extract_policy(&txt) {
                        match policy.as_str() {
                            "reject" | "quarantine" => {
                                return CheckResult::pass(
                                    CAT,
                                    "dmarc_policy",
                                    "DMARC policy set to quarantine or reject",
                                    3,
                                    Some(format!("p={policy}")),
                                );
                            }
                            "none" => {
                                return CheckResult::fail(
                                    CAT,
                                    "dmarc_policy",
                                    "DMARC policy set to quarantine or reject",
                                    3,
                                    Some(
                                        "DMARC policy is \"none\" (should be \"quarantine\" or \"reject\")"
                                            .into(),
                                    ),
                                );
                            }
                            other => {
                                return CheckResult::fail(
                                    CAT,
                                    "dmarc_policy",
                                    "DMARC policy set to quarantine or reject",
                                    3,
                                    Some(format!("Unknown DMARC policy: \"{other}\"")),
                                );
                            }
                        }
                    }
                }
            }
            CheckResult::fail(
                CAT,
                "dmarc_policy",
                "DMARC policy set to quarantine or reject",
                3,
                Some("No DMARC record found".into()),
            )
        }
        Err(e) => CheckResult::fail(
            CAT,
            "dmarc_policy",
            "DMARC policy set to quarantine or reject",
            3,
            Some(format!("DNS lookup failed: {e}")),
        ),
    }
}

fn extract_policy(dmarc_record: &str) -> Option<String> {
    for part in dmarc_record.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("p=") {
            return Some(val.trim().to_lowercase());
        }
    }
    None
}

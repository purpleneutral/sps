use hickory_resolver::TokioResolver;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::EmailDnsSecurity;

/// Check SPF record: present and ends with -all (hard fail).
pub async fn check_spf(domain: &str, resolver: &TokioResolver) -> CheckResult {
    match resolver.txt_lookup(domain).await {
        Ok(response) => {
            for record in response.iter() {
                let txt = record.to_string();
                if txt.starts_with("v=spf1") {
                    if txt.contains("-all") {
                        return CheckResult::pass(
                            CAT,
                            "spf_present",
                            "SPF record present and strict",
                            3,
                            Some(txt),
                        );
                    } else if txt.contains("~all") {
                        return CheckResult::fail(
                            CAT,
                            "spf_present",
                            "SPF record present and strict",
                            3,
                            Some(format!("{txt} (uses ~all softfail, should be -all)")),
                        );
                    } else {
                        return CheckResult::fail(
                            CAT,
                            "spf_present",
                            "SPF record present and strict",
                            3,
                            Some(format!("{txt} (does not end with -all)")),
                        );
                    }
                }
            }
            CheckResult::fail(
                CAT,
                "spf_present",
                "SPF record present and strict",
                3,
                Some("No SPF record found in TXT records".into()),
            )
        }
        Err(e) => CheckResult::fail(
            CAT,
            "spf_present",
            "SPF record present and strict",
            3,
            Some(format!("DNS lookup failed: {e}")),
        ),
    }
}

use reqwest::header::HeaderMap;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::SecurityHeaders;

const ACCEPTABLE_POLICIES: &[&str] = &[
    "no-referrer",
    "same-origin",
    "strict-origin",
    "strict-origin-when-cross-origin",
];

/// Check Referrer-Policy header.
pub fn check_referrer_policy(headers: &HeaderMap) -> CheckResult {
    let value = headers
        .get("referrer-policy")
        .and_then(|v| v.to_str().ok());

    match value {
        Some(policy) => {
            let policy_lower = policy.to_lowercase();
            // The header can contain multiple comma-separated policies; the last valid one wins.
            let effective = policy_lower
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .last()
                .unwrap_or("");

            if ACCEPTABLE_POLICIES.contains(&effective) {
                CheckResult::pass(
                    CAT,
                    "referrer_policy",
                    "Referrer-Policy set",
                    3,
                    Some(format!("Referrer-Policy: {policy}")),
                )
            } else {
                CheckResult::fail(
                    CAT,
                    "referrer_policy",
                    "Referrer-Policy set",
                    3,
                    Some(format!(
                        "Referrer-Policy: {policy} (not restrictive enough)"
                    )),
                )
            }
        }
        None => CheckResult::fail(
            CAT,
            "referrer_policy",
            "Referrer-Policy set",
            3,
            Some("Referrer-Policy header not present".into()),
        ),
    }
}

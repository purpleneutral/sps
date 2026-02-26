use reqwest::header::HeaderMap;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::SecurityHeaders;

/// Sensitive browser APIs that should be restricted.
const SENSITIVE_FEATURES: &[&str] = &[
    "camera",
    "microphone",
    "geolocation",
    "accelerometer",
    "gyroscope",
    "magnetometer",
    "payment",
    "usb",
    "bluetooth",
];

/// Check Permissions-Policy header.
pub fn check_permissions_policy(headers: &HeaderMap) -> CheckResult {
    let value = headers
        .get("permissions-policy")
        .and_then(|v| v.to_str().ok());

    match value {
        Some(policy) => {
            // Count how many sensitive features are restricted (set to () meaning "none")
            let restricted: Vec<&str> = SENSITIVE_FEATURES
                .iter()
                .filter(|feature| {
                    // Common restrictive patterns: feature=(), feature=self
                    let pattern_none = format!("{feature}=()");
                    let pattern_self = format!("{feature}=(self)");
                    policy.contains(&pattern_none) || policy.contains(&pattern_self)
                })
                .copied()
                .collect();

            if !restricted.is_empty() {
                CheckResult::pass(
                    CAT,
                    "permissions_policy",
                    "Permissions-Policy set",
                    3,
                    Some(format!(
                        "Restricts {} sensitive API(s): {}",
                        restricted.len(),
                        restricted.join(", ")
                    )),
                )
            } else {
                CheckResult::fail(
                    CAT,
                    "permissions_policy",
                    "Permissions-Policy set",
                    3,
                    Some("Permissions-Policy present but does not restrict sensitive APIs".into()),
                )
            }
        }
        None => CheckResult::fail(
            CAT,
            "permissions_policy",
            "Permissions-Policy set",
            3,
            Some("Permissions-Policy header not present".into()),
        ),
    }
}

use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::TransportSecurity;
const ONE_YEAR_SECS: u64 = 31_536_000;

/// Check HSTS header presence and directives.
///
/// Returns a vec of check results for:
/// - HSTS enabled
/// - HSTS max-age >= 1 year
/// - HSTS includeSubDomains
/// - HSTS preload
pub async fn check_hsts(domain: &str) -> Vec<CheckResult> {
    let mut checks = Vec::new();

    let hsts_value = fetch_hsts_header(domain).await;

    match hsts_value {
        Some(value) => {
            checks.push(CheckResult::pass(
                CAT,
                "hsts_enabled",
                "HSTS enabled",
                4,
                Some(format!("Strict-Transport-Security: {value}")),
            ));

            let directives = parse_hsts(&value);

            // max-age >= 1 year
            if let Some(max_age) = directives.max_age {
                if max_age >= ONE_YEAR_SECS {
                    checks.push(CheckResult::pass(
                        CAT,
                        "hsts_max_age",
                        "HSTS max-age >= 1 year",
                        2,
                        Some(format!("max-age={max_age}")),
                    ));
                } else {
                    checks.push(CheckResult::fail(
                        CAT,
                        "hsts_max_age",
                        "HSTS max-age >= 1 year",
                        2,
                        Some(format!("max-age={max_age} (less than {ONE_YEAR_SECS})")),
                    ));
                }
            } else {
                checks.push(CheckResult::fail(
                    CAT,
                    "hsts_max_age",
                    "HSTS max-age >= 1 year",
                    2,
                    Some("max-age directive not found".into()),
                ));
            }

            // includeSubDomains
            if directives.include_subdomains {
                checks.push(CheckResult::pass(
                    CAT,
                    "hsts_subdomains",
                    "HSTS includeSubDomains",
                    1,
                    None,
                ));
            } else {
                checks.push(CheckResult::fail(
                    CAT,
                    "hsts_subdomains",
                    "HSTS includeSubDomains",
                    1,
                    Some("includeSubDomains not set".into()),
                ));
            }

            // preload
            if directives.preload {
                checks.push(CheckResult::pass(
                    CAT,
                    "hsts_preload",
                    "HSTS preload",
                    1,
                    None,
                ));
            } else {
                checks.push(CheckResult::fail(
                    CAT,
                    "hsts_preload",
                    "HSTS preload",
                    1,
                    Some("preload directive not declared".into()),
                ));
            }
        }
        None => {
            checks.push(CheckResult::fail(
                CAT,
                "hsts_enabled",
                "HSTS enabled",
                4,
                Some("Strict-Transport-Security header not present".into()),
            ));
            checks.push(CheckResult::fail(
                CAT,
                "hsts_max_age",
                "HSTS max-age >= 1 year",
                2,
                None,
            ));
            checks.push(CheckResult::fail(
                CAT,
                "hsts_subdomains",
                "HSTS includeSubDomains",
                1,
                None,
            ));
            checks.push(CheckResult::fail(
                CAT,
                "hsts_preload",
                "HSTS preload",
                1,
                None,
            ));
        }
    }

    checks
}

async fn fetch_hsts_header(domain: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()?;

    let url = format!("https://{domain}");
    let resp = client.get(&url).send().await.ok()?;

    resp.headers()
        .get("strict-transport-security")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

struct HstsDirectives {
    max_age: Option<u64>,
    include_subdomains: bool,
    preload: bool,
}

fn parse_hsts(value: &str) -> HstsDirectives {
    let lower = value.to_lowercase();
    let mut max_age = None;
    let mut include_subdomains = false;
    let mut preload = false;

    for part in lower.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("max-age=") {
            max_age = val.trim().parse().ok();
        } else if part == "includesubdomains" {
            include_subdomains = true;
        } else if part == "preload" {
            preload = true;
        }
    }

    HstsDirectives {
        max_age,
        include_subdomains,
        preload,
    }
}

use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::BestPractices;

/// Check for a valid security.txt at /.well-known/security.txt (RFC 9116).
pub async fn check_security_txt(domain: &str) -> CheckResult {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 3 {
                attempt.stop()
            } else if let Some(host) = attempt.url().host_str() {
                if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                    if scanner_core::ssrf::is_private_ip(&ip) {
                        attempt.stop()
                    } else {
                        attempt.follow()
                    }
                } else {
                    attempt.follow()
                }
            } else {
                attempt.follow()
            }
        }))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return CheckResult::fail(
                CAT,
                "security_txt",
                "security.txt present",
                2,
                Some(format!("HTTP client error: {e}")),
            );
        }
    };

    let url = format!("https://{domain}/.well-known/security.txt");

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            let issues = validate_security_txt(&body);

            if issues.is_empty() {
                CheckResult::pass(
                    CAT,
                    "security_txt",
                    "security.txt present",
                    2,
                    Some("Present and valid per RFC 9116".into()),
                )
            } else {
                CheckResult::pass(
                    CAT,
                    "security_txt",
                    "security.txt present",
                    2,
                    Some(format!("Present but with issues: {}", issues.join("; "))),
                )
            }
        }
        Ok(resp) => CheckResult::fail(
            CAT,
            "security_txt",
            "security.txt present",
            2,
            Some(format!(
                "/.well-known/security.txt returned HTTP {}",
                resp.status()
            )),
        ),
        Err(e) => CheckResult::fail(
            CAT,
            "security_txt",
            "security.txt present",
            2,
            Some(format!("Request failed: {e}")),
        ),
    }
}

/// Basic RFC 9116 validation.
fn validate_security_txt(body: &str) -> Vec<String> {
    let mut issues = Vec::new();

    let has_contact = body.lines().any(|l| {
        let l = l.trim();
        l.starts_with("Contact:") || l.starts_with("contact:")
    });
    if !has_contact {
        issues.push("Missing required Contact field".into());
    }

    let has_expires = body.lines().any(|l| {
        let l = l.trim();
        l.starts_with("Expires:") || l.starts_with("expires:")
    });
    if !has_expires {
        issues.push("Missing recommended Expires field".into());
    }

    issues
}

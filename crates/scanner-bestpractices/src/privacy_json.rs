use scanner_core::check::CheckResult;
use scanner_core::spec::Category;
use serde::Deserialize;

const CAT: Category = Category::BestPractices;

/// Expected structure of /.well-known/privacy.json.
#[derive(Debug, Deserialize)]
struct PrivacyJson {
    spec: Option<String>,
    version: Option<String>,
    claims: Option<serde_json::Value>,
    contact: Option<String>,
    #[allow(dead_code)]
    last_reviewed: Option<String>,
}

/// Check for presence of /.well-known/privacy.json (Seglamater-proposed standard).
pub async fn check_privacy_json(domain: &str) -> CheckResult {
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
                "privacy_json",
                "privacy.json present",
                2,
                Some(format!("HTTP client error: {e}")),
            );
        }
    };

    let url = format!("https://{domain}/.well-known/privacy.json");

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            match serde_json::from_str::<PrivacyJson>(&body) {
                Ok(pj) => {
                    let mut details = vec!["Valid JSON".to_string()];
                    if let Some(spec) = &pj.spec {
                        details.push(format!("spec: {spec}"));
                    }
                    if let Some(ver) = &pj.version {
                        details.push(format!("version: {ver}"));
                    }
                    if pj.claims.is_some() {
                        details.push("claims present".to_string());
                    }
                    if let Some(contact) = &pj.contact {
                        details.push(format!("contact: {contact}"));
                    }

                    CheckResult::pass(
                        CAT,
                        "privacy_json",
                        "privacy.json present",
                        2,
                        Some(details.join(", ")),
                    )
                }
                Err(e) => CheckResult::fail(
                    CAT,
                    "privacy_json",
                    "privacy.json present",
                    2,
                    Some(format!("File exists but is not valid JSON: {e}")),
                ),
            }
        }
        Ok(resp) => CheckResult::fail(
            CAT,
            "privacy_json",
            "privacy.json present",
            2,
            Some(format!(
                "/.well-known/privacy.json returned HTTP {}",
                resp.status()
            )),
        ),
        Err(e) => CheckResult::fail(
            CAT,
            "privacy_json",
            "privacy.json present",
            2,
            Some(format!("Request failed: {e}")),
        ),
    }
}

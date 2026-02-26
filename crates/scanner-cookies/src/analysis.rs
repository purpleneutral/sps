/// Parsed information about a single Set-Cookie header.
#[derive(Debug, Clone)]
pub struct CookieInfo {
    pub name: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
    pub max_age_seconds: Option<i64>,
    pub domain: Option<String>,
    pub is_third_party: bool,
}

/// Parse a raw Set-Cookie header value into structured info.
pub fn parse_set_cookie(raw: &str, first_party_domain: &str) -> CookieInfo {
    let parts: Vec<&str> = raw.split(';').collect();

    // First part is name=value
    let name = parts
        .first()
        .and_then(|p| p.split('=').next())
        .unwrap_or("unknown")
        .trim()
        .to_string();

    let lower = raw.to_lowercase();
    let secure = lower.contains("; secure") || lower.contains(";secure");
    let http_only = lower.contains("; httponly") || lower.contains(";httponly");

    let same_site = extract_attribute(&lower, "samesite");
    let max_age = extract_attribute(&lower, "max-age").and_then(|v| v.parse::<i64>().ok());
    let cookie_domain = extract_attribute(&lower, "domain");

    // Check for expires if no max-age
    let max_age_seconds = max_age.or_else(|| parse_expires_to_seconds(&lower));

    let is_third_party = cookie_domain
        .as_ref()
        .is_some_and(|d| !is_same_domain(d, first_party_domain));

    CookieInfo {
        name,
        secure,
        http_only,
        same_site,
        max_age_seconds,
        domain: cookie_domain,
        is_third_party,
    }
}

fn extract_attribute(lower_raw: &str, attr: &str) -> Option<String> {
    for part in lower_raw.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix(&format!("{attr}=")) {
            return Some(val.trim().to_string());
        }
    }
    None
}

fn parse_expires_to_seconds(lower_raw: &str) -> Option<i64> {
    let expires_str = extract_attribute(lower_raw, "expires")?;
    let expires = chrono::DateTime::parse_from_rfc2822(&expires_str).ok()?;
    let now = chrono::Utc::now();
    let duration = expires.signed_duration_since(now);
    Some(duration.num_seconds())
}

fn is_same_domain(cookie_domain: &str, first_party: &str) -> bool {
    let cd = cookie_domain.trim_start_matches('.').to_lowercase();
    let fp = first_party.to_lowercase();

    cd == fp || fp.ends_with(&format!(".{cd}")) || cd.ends_with(&format!(".{fp}"))
}

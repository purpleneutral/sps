use crate::blocklists::{self, CDN_DOMAINS};
use crate::parser::ExternalResource;

/// Find analytics scripts among external resources.
/// Returns list of analytics domains found.
pub fn find_analytics(resources: &[ExternalResource]) -> Vec<String> {
    let mut found = Vec::new();
    for r in resources {
        if blocklists::domain_matches_list(&r.domain, blocklists::ANALYTICS_DOMAINS)
            && !found.contains(&r.domain)
        {
            found.push(r.domain.clone());
        }
    }
    found
}

/// Find advertising/tracking scripts among external resources.
/// Returns list of tracker domains found.
pub fn find_trackers(resources: &[ExternalResource]) -> Vec<String> {
    let mut found = Vec::new();
    for r in resources {
        if blocklists::domain_matches_list(&r.domain, blocklists::TRACKER_DOMAINS)
            && !found.contains(&r.domain)
        {
            found.push(r.domain.clone());
        }
    }
    found
}

/// Find third-party CDN resources (fonts, scripts not from first-party domain).
/// Returns list of CDN domains found.
pub fn find_third_party_cdns(resources: &[ExternalResource], first_party: &str) -> Vec<String> {
    let mut found = Vec::new();
    for r in resources {
        if is_third_party(&r.domain, first_party)
            && blocklists::domain_matches_list(&r.domain, CDN_DOMAINS)
            && !found.contains(&r.domain)
        {
            found.push(r.domain.clone());
        }
    }
    found
}

/// Check if any external resource uses HTTP instead of HTTPS.
pub fn has_mixed_content(resources: &[ExternalResource]) -> bool {
    resources.iter().any(|r| !r.is_https)
}

/// Check if a resource domain is third-party relative to the scanned domain.
fn is_third_party(resource_domain: &str, first_party: &str) -> bool {
    let rd = resource_domain.to_lowercase();
    let fp = first_party.to_lowercase();

    if rd == fp {
        return false;
    }

    // Check if resource is a subdomain of first-party or vice versa
    if rd.ends_with(&format!(".{fp}")) || fp.ends_with(&format!(".{rd}")) {
        return false;
    }

    true
}

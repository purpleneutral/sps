use anyhow::{Result, bail};
use hickory_resolver::TokioResolver;
use std::net::IpAddr;
use std::time::Duration;

// Re-export from scanner-core for backward compatibility
pub use scanner_core::ssrf::is_private_ip;

/// Timeout for DNS resolution to prevent indefinite hangs.
const DNS_TIMEOUT: Duration = Duration::from_secs(5);

/// Cloud metadata and internal hostnames that must never be scanned.
const BLOCKED_HOSTNAMES: &[&str] = &[
    "metadata.google.internal",
    "metadata.google.com",
    "169.254.169.254",
    "metadata.internal",
];

/// Maximum domain length per RFC 1035.
const MAX_DOMAIN_LENGTH: usize = 253;

/// Validate that a domain is safe to scan.
///
/// Rejects bare IP addresses, invalid formats, cloud metadata endpoints,
/// and domains that resolve to private/reserved IP ranges.
pub async fn validate_domain(domain: &str) -> Result<()> {
    if domain.is_empty() {
        bail!("Domain is empty");
    }

    if domain.len() > MAX_DOMAIN_LENGTH {
        bail!("Domain exceeds maximum length of {MAX_DOMAIN_LENGTH} characters");
    }

    // Reject bare IP addresses
    if domain.parse::<IpAddr>().is_ok() {
        bail!("Bare IP addresses are not allowed — provide a domain name");
    }
    if domain.starts_with('[') {
        bail!("Bare IP addresses are not allowed — provide a domain name");
    }

    validate_domain_format(domain)?;

    // Reject known cloud metadata hostnames
    let lower = domain.to_lowercase();
    for blocked in BLOCKED_HOSTNAMES {
        if lower == *blocked {
            bail!("Domain is blocked");
        }
    }

    // Resolve and reject private/reserved IPs
    validate_resolved_ips(domain).await?;

    Ok(())
}

/// Validate basic FQDN format: alphanumeric labels separated by dots.
fn validate_domain_format(domain: &str) -> Result<()> {
    let labels: Vec<&str> = domain.split('.').collect();

    if labels.len() < 2 {
        bail!("Invalid domain — must be a fully qualified domain name");
    }

    for label in &labels {
        if label.is_empty() || label.len() > 63 {
            bail!("Invalid domain label length");
        }
        if label.starts_with('-') || label.ends_with('-') {
            bail!("Invalid domain — labels cannot start or end with a hyphen");
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            bail!("Invalid domain — only ASCII alphanumeric characters and hyphens allowed");
        }
    }

    Ok(())
}

/// Resolve the domain and reject if any IP is private or reserved.
async fn validate_resolved_ips(domain: &str) -> Result<()> {
    let resolver = TokioResolver::builder_tokio()
        .map_err(|_| anyhow::anyhow!("Failed to create DNS resolver"))?
        .build();

    let ips: Vec<IpAddr> = tokio::time::timeout(DNS_TIMEOUT, resolver.lookup_ip(domain))
        .await
        .map_err(|_| anyhow::anyhow!("DNS resolution timed out for domain"))?
        .map_err(|_| anyhow::anyhow!("DNS resolution failed for domain"))?
        .iter()
        .collect();

    if ips.is_empty() {
        bail!("Domain did not resolve to any IP addresses");
    }

    for ip in &ips {
        if is_private_ip(ip) {
            bail!("Domain resolves to a private or reserved IP address");
        }
    }

    Ok(())
}

use anyhow::{bail, Result};
use hickory_resolver::TokioResolver;
use std::net::{IpAddr, Ipv4Addr};

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

    let ips: Vec<IpAddr> = resolver
        .lookup_ip(domain)
        .await
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

/// Check whether an IP address is in a private, loopback, link-local,
/// or otherwise reserved range that should not be scanned.
pub fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()                // 127.0.0.0/8
            || v4.is_private()              // 10/8, 172.16/12, 192.168/16
            || v4.is_link_local()           // 169.254.0.0/16
            || v4.is_broadcast()            // 255.255.255.255
            || v4.is_unspecified()          // 0.0.0.0
            || is_cgnat(v4)                 // 100.64.0.0/10
            || is_benchmarking(v4)          // 198.18.0.0/15
            || is_documentation(v4)         // 192.0.2/24, 198.51.100/24, 203.0.113/24
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()                // ::1
            || v6.is_unspecified()          // ::
            || is_v6_link_local(v6)         // fe80::/10
            || is_v6_unique_local(v6)       // fc00::/7
            || is_v4_mapped_private(v6)     // ::ffff:private
        }
    }
}

fn is_cgnat(v4: &Ipv4Addr) -> bool {
    let o = v4.octets();
    o[0] == 100 && (o[1] & 0xC0) == 64 // 100.64.0.0/10
}

fn is_benchmarking(v4: &Ipv4Addr) -> bool {
    let o = v4.octets();
    o[0] == 198 && (o[1] == 18 || o[1] == 19) // 198.18.0.0/15
}

fn is_documentation(v4: &Ipv4Addr) -> bool {
    let o = v4.octets();
    (o[0] == 192 && o[1] == 0 && o[2] == 2)       // 192.0.2.0/24 (TEST-NET-1)
    || (o[0] == 198 && o[1] == 51 && o[2] == 100)  // 198.51.100.0/24 (TEST-NET-2)
    || (o[0] == 203 && o[1] == 0 && o[2] == 113)   // 203.0.113.0/24 (TEST-NET-3)
}

fn is_v6_link_local(v6: &std::net::Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xffc0) == 0xfe80
}

fn is_v6_unique_local(v6: &std::net::Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xfe00) == 0xfc00
}

fn is_v4_mapped_private(v6: &std::net::Ipv6Addr) -> bool {
    let s = v6.segments();
    if s[0] == 0 && s[1] == 0 && s[2] == 0 && s[3] == 0 && s[4] == 0 && s[5] == 0xffff {
        let v4 = Ipv4Addr::new((s[6] >> 8) as u8, s[6] as u8, (s[7] >> 8) as u8, s[7] as u8);
        is_private_ip(&IpAddr::V4(v4))
    } else {
        false
    }
}

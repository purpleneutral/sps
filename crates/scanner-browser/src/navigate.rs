use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::network::{
    EventRequestWillBeSent, EventResponseReceived,
};
use futures::StreamExt;
use scanner_core::browser_types::NetworkRequest;
use scanner_core::ssrf::is_private_ip;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

pub type NetworkLog = Arc<Mutex<Vec<NetworkRequest>>>;

/// Validate that the target domain resolves to public IPs only.
/// Returns the first public IP for DNS pinning.
pub async fn validate_target(domain: &str) -> Result<String> {
    let resolver = hickory_resolver::TokioResolver::builder_tokio()
        .map_err(|_| anyhow::anyhow!("Failed to create DNS resolver"))?
        .build();

    let ips: Vec<IpAddr> = resolver
        .lookup_ip(domain)
        .await
        .map_err(|_| anyhow::anyhow!("DNS resolution failed for {domain}"))?
        .iter()
        .collect();

    if ips.is_empty() {
        bail!("Domain did not resolve to any IP addresses");
    }

    let public_ip = ips
        .iter()
        .find(|ip| !is_private_ip(ip))
        .ok_or_else(|| anyhow::anyhow!("Domain resolves to private IPs only"))?;

    Ok(public_ip.to_string())
}

/// Set up CDP network event listeners to capture all requests.
pub async fn setup_network_interception(page: &Page) -> Result<NetworkLog> {
    let log: NetworkLog = Arc::new(Mutex::new(Vec::new()));

    // Listen for outgoing requests
    let log_clone = log.clone();
    let mut request_events = page.event_listener::<EventRequestWillBeSent>().await?;

    tokio::spawn(async move {
        while let Some(event) = request_events.next().await {
            let url_str = event.request.url.clone();

            let domain = Url::parse(&url_str)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default();
            let is_https = url_str.starts_with("https://");

            let req = NetworkRequest {
                url: url_str,
                domain,
                method: event.request.method.clone(),
                resource_type: format!("{:?}", event.r#type),
                status: 0,
                mime_type: None,
                initiator: format!("{:?}", event.initiator.r#type),
                is_https,
            };

            log_clone.lock().await.push(req);
        }
    });

    // Listen for responses to fill in status codes and MIME types
    let log_clone = log.clone();
    let mut response_events = page.event_listener::<EventResponseReceived>().await?;

    tokio::spawn(async move {
        while let Some(event) = response_events.next().await {
            let url = event.response.url.clone();
            let status = event.response.status as u16;
            let mime = event.response.mime_type.clone();

            let mut entries = log_clone.lock().await;
            if let Some(entry) = entries.iter_mut().rev().find(|r| r.url == url) {
                entry.status = status;
                entry.mime_type = Some(mime);
            }
        }
    });

    Ok(log)
}

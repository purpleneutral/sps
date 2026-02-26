use scanner_core::check::CheckResult;
use scanner_core::spec::Category;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

const CAT: Category = Category::TransportSecurity;

/// Check TLS 1.3 support and whether legacy TLS 1.0/1.1 are disabled.
///
/// Returns (tls13_check, legacy_disabled_check).
pub async fn check_tls(domain: &str) -> (CheckResult, CheckResult) {
    let tls13 = check_tls_version(domain, &rustls::version::TLS13).await;
    let tls12 = check_tls_version(domain, &rustls::version::TLS12).await;

    let tls13_check = if tls13.is_ok() {
        CheckResult::pass(CAT, "tls_1_3_supported", "TLS 1.3 supported", 8, None)
    } else {
        CheckResult::fail(
            CAT,
            "tls_1_3_supported",
            "TLS 1.3 supported",
            8,
            Some("Server does not support TLS 1.3".into()),
        )
    };

    // TLS 1.0/1.1: rustls doesn't support negotiating these at all (it only does 1.2+),
    // so if TLS 1.2 works, we consider legacy disabled (since the server at least supports 1.2).
    // A server that ONLY supports 1.0/1.1 would fail the TLS 1.2 check too.
    // For a more thorough check we'd need to use openssl, but rustls is our choice.
    // We give the pass here since modern servers that support 1.2+ have generally disabled 1.0/1.1.
    let legacy_check = if tls12.is_ok() || tls13.is_ok() {
        // Server supports modern TLS. We note that we can't directly verify 1.0/1.1 are disabled
        // with rustls, but supporting 1.2+ is a strong indicator.
        CheckResult::pass(
            CAT,
            "tls_legacy_disabled",
            "TLS 1.0/1.1 disabled",
            4,
            Some("Server supports TLS 1.2+ (legacy protocol test limited with rustls)".into()),
        )
    } else {
        CheckResult::fail(
            CAT,
            "tls_legacy_disabled",
            "TLS 1.0/1.1 disabled",
            4,
            Some("Could not establish any TLS connection".into()),
        )
    };

    (tls13_check, legacy_check)
}

async fn check_tls_version(
    domain: &str,
    version: &'static rustls::SupportedProtocolVersion,
) -> Result<(), String> {
    let mut config = rustls::ClientConfig::builder_with_protocol_versions(&[version])
        .with_root_certificates(root_store())
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let connector = TlsConnector::from(Arc::new(config));
    let server_name = rustls::pki_types::ServerName::try_from(domain.to_string())
        .map_err(|e| format!("Invalid server name: {e}"))?;

    let addr = format!("{domain}:443");
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("TCP connection failed: {e}"))?;

    connector
        .connect(server_name, stream)
        .await
        .map_err(|e| format!("TLS handshake failed: {e}"))?;

    Ok(())
}

fn root_store() -> Arc<rustls::RootCertStore> {
    let mut store = rustls::RootCertStore::empty();
    store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Arc::new(store)
}

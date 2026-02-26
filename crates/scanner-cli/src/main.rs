use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use scanner_core::check::CategoryResult;
use scanner_core::report;
use scanner_core::score::ScanResult;
use std::time::Instant;

mod recommendations;

/// Seglamater Privacy Scanner — evaluate any website against the Seglamater Privacy Specification
#[derive(Parser, Debug)]
#[command(name = "seglamater-scan", version, about)]
struct Args {
    /// Domain to scan (e.g., example.com)
    domain: String,

    /// Output format
    #[arg(long, default_value = "text", value_parser = ["text", "json"])]
    format: String,

    /// Specification version to use
    #[arg(long, default_value = "v1.0")]
    spec: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install the rustls crypto provider (aws-lc-rs) before any TLS operations
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("seglamater=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();
    let domain = normalize_domain(&args.domain);

    if args.format == "text" {
        eprintln!(
            "{} {}",
            "Scanning".green().bold(),
            domain.bold()
        );
        eprintln!();
    }

    let start = Instant::now();
    let result = run_scan(&domain).await?;
    let elapsed = start.elapsed();

    match args.format.as_str() {
        "json" => println!("{}", report::format_json(&result)),
        _ => {
            println!("{}", report::format_text(&result));
            eprintln!(
                "{}",
                format!("Scan completed in {:.1}s", elapsed.as_secs_f64()).dimmed()
            );
        }
    }

    Ok(())
}

async fn run_scan(domain: &str) -> Result<ScanResult> {
    // Fetch the page HTML and headers once, share across checks
    let (headers, html, set_cookie_headers) = fetch_page(domain).await?;

    let page_url = format!("https://{domain}/");

    // Run checks in parallel where possible
    let (transport, dns) = tokio::join!(
        scanner_transport::check_transport(domain),
        scanner_dns::check_dns(domain),
    );

    // These depend on the fetched page data
    let security_headers = scanner_headers::check_headers(&headers);
    let tracking = scanner_tracking::check_tracking(domain, &html, &page_url);
    let cookies = scanner_cookies::check_cookies(&set_cookie_headers, domain);

    let best_practices = scanner_bestpractices::check_best_practices(domain, &html).await;

    let categories: Vec<CategoryResult> = vec![
        transport,
        security_headers,
        tracking,
        cookies,
        dns,
        best_practices,
    ];

    let recs = recommendations::generate(&categories);

    Ok(ScanResult::from_categories(
        domain.to_string(),
        categories,
        recs,
    ))
}

/// Fetch the page and extract headers, HTML body, and Set-Cookie headers.
async fn fetch_page(
    domain: &str,
) -> Result<(reqwest::header::HeaderMap, String, Vec<String>)> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (compatible; SeglamaterScan/0.1; +https://seglamater.com/scan)")
        .build()
        .context("Failed to build HTTP client")?;

    let url = format!("https://{domain}");
    let resp = client
        .get(&url)
        .send()
        .await
        .context(format!("Failed to connect to {domain}"))?;

    let headers = resp.headers().clone();

    let set_cookie_headers: Vec<String> = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .collect();

    let html = resp
        .text()
        .await
        .context("Failed to read response body")?;

    Ok((headers, html, set_cookie_headers))
}

/// Normalize domain input: strip protocol, trailing slashes, paths.
fn normalize_domain(input: &str) -> String {
    let mut domain = input.trim().to_string();

    // Strip protocol
    if let Some(rest) = domain.strip_prefix("https://") {
        domain = rest.to_string();
    } else if let Some(rest) = domain.strip_prefix("http://") {
        domain = rest.to_string();
    }

    // Strip path and trailing slash
    if let Some(idx) = domain.find('/') {
        domain = domain[..idx].to_string();
    }

    // Strip port
    if let Some(idx) = domain.find(':') {
        domain = domain[..idx].to_string();
    }

    domain.to_lowercase()
}

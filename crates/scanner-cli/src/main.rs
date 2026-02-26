use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use scanner_core::report;
use scanner_engine::normalize_domain;
use std::time::Instant;

/// Seglamater Privacy Scanner — evaluate any website against the Seglamater Privacy Specification
#[derive(Parser, Debug)]
#[command(name = "seglamater-scan", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan a domain against the Seglamater Privacy Specification
    Scan {
        /// Domain to scan (e.g., example.com)
        domain: String,

        /// Output format
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
    },

    /// Start the scanner API server
    Serve {
        /// Address to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Database URL (SQLite: sqlite://scanner.db, PostgreSQL: postgres://user:pass@host/db)
        #[arg(long, env = "DATABASE_URL", default_value = "sqlite://./scanner.db")]
        database_url: String,
    },
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

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { domain, format } => cmd_scan(&domain, &format).await,
        Commands::Serve {
            host,
            port,
            database_url,
        } => cmd_serve(&host, port, &database_url).await,
    }
}

async fn cmd_scan(domain: &str, format: &str) -> Result<()> {
    let domain = normalize_domain(domain);

    if format == "text" {
        eprintln!("{} {}", "Scanning".green().bold(), domain.bold());
        eprintln!();
    }

    let start = Instant::now();
    let result = scanner_engine::run_scan(&domain).await?;
    let elapsed = start.elapsed();

    match format {
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

async fn cmd_serve(host: &str, port: u16, database_url: &str) -> Result<()> {
    scanner_server::run_server(host, port, database_url).await
}

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A stored scan record (summary, without full check details).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRecord {
    pub id: i64,
    pub domain: String,
    pub score: u32,
    pub grade: String,
    pub scan_data: String,
    pub scanned_at: DateTime<Utc>,
}

/// A registered domain for scheduled re-scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDomain {
    pub domain: String,
    pub registered_at: DateTime<Utc>,
    pub scan_interval_hours: i32,
    pub last_scanned_at: Option<DateTime<Utc>>,
}

/// Aggregate statistics across all scans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateStats {
    pub total_domains: i64,
    pub total_scans: i64,
    pub average_score: f64,
    pub grade_distribution: GradeDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeDistribution {
    pub a_plus: i64,
    pub a: i64,
    pub b: i64,
    pub c: i64,
    pub d: i64,
    pub f: i64,
}

/// Storage backend abstraction for the scanner server.
///
/// Implement this trait to add support for a new database backend.
pub trait Storage: Send + Sync + 'static {
    /// Store a scan result. Returns the record ID.
    fn store_scan(
        &self,
        domain: &str,
        score: u32,
        grade: &str,
        scan_data: &str,
    ) -> impl std::future::Future<Output = Result<i64>> + Send;

    /// Get the latest scan for a domain.
    fn get_latest_scan(
        &self,
        domain: &str,
    ) -> impl std::future::Future<Output = Result<Option<ScanRecord>>> + Send;

    /// Get scan history for a domain (most recent first).
    fn get_history(
        &self,
        domain: &str,
        limit: i64,
    ) -> impl std::future::Future<Output = Result<Vec<ScanRecord>>> + Send;

    /// List all scanned domains with their latest score.
    fn list_domains(
        &self,
        limit: i64,
        offset: i64,
    ) -> impl std::future::Future<Output = Result<Vec<ScanRecord>>> + Send;

    /// Search domains by prefix.
    fn search_domains(
        &self,
        query: &str,
        limit: i64,
    ) -> impl std::future::Future<Output = Result<Vec<ScanRecord>>> + Send;

    /// Register a domain for scheduled re-scanning.
    fn register_domain(
        &self,
        domain: &str,
        interval_hours: i32,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get domains that are due for a re-scan.
    fn get_due_domains(&self) -> impl std::future::Future<Output = Result<Vec<String>>> + Send;

    /// Update the last_scanned_at timestamp for a registered domain.
    fn mark_scanned(&self, domain: &str) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get aggregate statistics.
    fn get_stats(&self) -> impl std::future::Future<Output = Result<AggregateStats>> + Send;
}

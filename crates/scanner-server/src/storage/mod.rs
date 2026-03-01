mod traits;

#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "postgres")]
mod postgres;

pub use traits::*;

#[cfg(feature = "sqlite")]
pub use sqlite::SqliteStorage;

#[cfg(feature = "postgres")]
pub use postgres::PostgresStorage;

use anyhow::Result;

/// Runtime-selected storage backend.
///
/// This enum dispatches to the correct implementation based on the database URL
/// provided at startup. Using an enum instead of `dyn Storage` because the
/// `Storage` trait uses `impl Future` return types (not object-safe).
pub enum AnyStorage {
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteStorage),
    #[cfg(feature = "postgres")]
    Postgres(PostgresStorage),
}

impl Storage for AnyStorage {
    async fn store_scan(
        &self,
        domain: &str,
        score: u32,
        grade: &str,
        scan_data: &str,
    ) -> Result<i64> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.store_scan(domain, score, grade, scan_data).await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.store_scan(domain, score, grade, scan_data).await,
        }
    }

    async fn get_latest_scan(&self, domain: &str) -> Result<Option<ScanRecord>> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.get_latest_scan(domain).await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.get_latest_scan(domain).await,
        }
    }

    async fn get_history(&self, domain: &str, limit: i64) -> Result<Vec<ScanRecord>> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.get_history(domain, limit).await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.get_history(domain, limit).await,
        }
    }

    async fn list_domains(&self, limit: i64, offset: i64) -> Result<Vec<ScanRecord>> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.list_domains(limit, offset).await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.list_domains(limit, offset).await,
        }
    }

    async fn search_domains(&self, query: &str, limit: i64) -> Result<Vec<ScanRecord>> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.search_domains(query, limit).await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.search_domains(query, limit).await,
        }
    }

    async fn register_domain(&self, domain: &str, interval_hours: i32) -> Result<()> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.register_domain(domain, interval_hours).await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.register_domain(domain, interval_hours).await,
        }
    }

    async fn get_due_domains(&self) -> Result<Vec<String>> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.get_due_domains().await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.get_due_domains().await,
        }
    }

    async fn mark_scanned(&self, domain: &str) -> Result<()> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.mark_scanned(domain).await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.mark_scanned(domain).await,
        }
    }

    async fn get_stats(&self) -> Result<AggregateStats> {
        match self {
            #[cfg(feature = "sqlite")]
            AnyStorage::Sqlite(s) => s.get_stats().await,
            #[cfg(feature = "postgres")]
            AnyStorage::Postgres(s) => s.get_stats().await,
        }
    }
}

/// Create a storage backend from a database URL.
///
/// - `sqlite://...` → SqliteStorage (requires `sqlite` feature)
/// - `postgres://...` or `postgresql://...` → PostgresStorage (requires `postgres` feature)
pub async fn connect(database_url: &str) -> Result<AnyStorage> {
    if database_url.starts_with("sqlite://") {
        #[cfg(feature = "sqlite")]
        {
            let s = SqliteStorage::connect(database_url).await?;
            return Ok(AnyStorage::Sqlite(s));
        }
        #[cfg(not(feature = "sqlite"))]
        {
            anyhow::bail!("SQLite support not compiled in. Build with --features sqlite");
        }
    } else if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            let s = PostgresStorage::connect(database_url).await?;
            return Ok(AnyStorage::Postgres(s));
        }
        #[cfg(not(feature = "postgres"))]
        {
            anyhow::bail!("PostgreSQL support not compiled in. Build with --features postgres");
        }
    }

    anyhow::bail!("Unsupported database URL scheme — expected sqlite:// or postgres://");
}

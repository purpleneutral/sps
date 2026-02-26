use super::{AggregateStats, GradeDistribution, ScanRecord, Storage};
use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await
            .context("Failed to connect to SQLite database")?;

        let storage = Self { pool };
        storage.run_migrations().await?;
        Ok(storage)
    }

    async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS scans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL,
                score INTEGER NOT NULL,
                grade TEXT NOT NULL,
                scan_data TEXT NOT NULL,
                scanned_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_scans_domain ON scans(domain)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_scans_domain_time ON scans(domain, scanned_at DESC)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS registered_domains (
                domain TEXT PRIMARY KEY,
                registered_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                scan_interval_hours INTEGER NOT NULL DEFAULT 24,
                last_scanned_at TEXT
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

impl Storage for SqliteStorage {
    async fn store_scan(
        &self,
        domain: &str,
        score: u32,
        grade: &str,
        scan_data: &str,
    ) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO scans (domain, score, grade, scan_data) VALUES (?, ?, ?, ?)",
        )
        .bind(domain)
        .bind(score as i64)
        .bind(grade)
        .bind(scan_data)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn get_latest_scan(&self, domain: &str) -> Result<Option<ScanRecord>> {
        let row = sqlx::query(
            "SELECT id, domain, score, grade, scan_data, scanned_at
             FROM scans WHERE domain = ? ORDER BY scanned_at DESC LIMIT 1",
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ScanRecord {
            id: r.get("id"),
            domain: r.get("domain"),
            score: r.get::<i64, _>("score") as u32,
            grade: r.get("grade"),
            scan_data: r.get("scan_data"),
            scanned_at: r.get("scanned_at"),
        }))
    }

    async fn get_history(&self, domain: &str, limit: i64) -> Result<Vec<ScanRecord>> {
        let rows = sqlx::query(
            "SELECT id, domain, score, grade, scan_data, scanned_at
             FROM scans WHERE domain = ? ORDER BY scanned_at DESC LIMIT ?",
        )
        .bind(domain)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ScanRecord {
                id: r.get("id"),
                domain: r.get("domain"),
                score: r.get::<i64, _>("score") as u32,
                grade: r.get("grade"),
                scan_data: r.get("scan_data"),
                scanned_at: r.get("scanned_at"),
            })
            .collect())
    }

    async fn list_domains(&self, limit: i64, offset: i64) -> Result<Vec<ScanRecord>> {
        let rows = sqlx::query(
            "SELECT s.id, s.domain, s.score, s.grade, s.scan_data, s.scanned_at
             FROM scans s
             INNER JOIN (
                 SELECT domain, MAX(scanned_at) as max_at FROM scans GROUP BY domain
             ) latest ON s.domain = latest.domain AND s.scanned_at = latest.max_at
             ORDER BY s.domain
             LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ScanRecord {
                id: r.get("id"),
                domain: r.get("domain"),
                score: r.get::<i64, _>("score") as u32,
                grade: r.get("grade"),
                scan_data: r.get("scan_data"),
                scanned_at: r.get("scanned_at"),
            })
            .collect())
    }

    async fn search_domains(&self, query: &str, limit: i64) -> Result<Vec<ScanRecord>> {
        let pattern = format!("{query}%");
        let rows = sqlx::query(
            "SELECT s.id, s.domain, s.score, s.grade, s.scan_data, s.scanned_at
             FROM scans s
             INNER JOIN (
                 SELECT domain, MAX(scanned_at) as max_at FROM scans GROUP BY domain
             ) latest ON s.domain = latest.domain AND s.scanned_at = latest.max_at
             WHERE s.domain LIKE ?
             ORDER BY s.domain
             LIMIT ?",
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ScanRecord {
                id: r.get("id"),
                domain: r.get("domain"),
                score: r.get::<i64, _>("score") as u32,
                grade: r.get("grade"),
                scan_data: r.get("scan_data"),
                scanned_at: r.get("scanned_at"),
            })
            .collect())
    }

    async fn register_domain(&self, domain: &str, interval_hours: i32) -> Result<()> {
        sqlx::query(
            "INSERT INTO registered_domains (domain, scan_interval_hours)
             VALUES (?, ?)
             ON CONFLICT(domain) DO UPDATE SET scan_interval_hours = excluded.scan_interval_hours",
        )
        .bind(domain)
        .bind(interval_hours)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_due_domains(&self) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT domain FROM registered_domains
             WHERE last_scanned_at IS NULL
                OR datetime(last_scanned_at, '+' || scan_interval_hours || ' hours') <= datetime('now')",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.get("domain")).collect())
    }

    async fn mark_scanned(&self, domain: &str) -> Result<()> {
        sqlx::query(
            "UPDATE registered_domains SET last_scanned_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE domain = ?",
        )
        .bind(domain)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_stats(&self) -> Result<AggregateStats> {
        let row = sqlx::query(
            "SELECT
                COUNT(DISTINCT domain) as total_domains,
                COUNT(*) as total_scans,
                COALESCE(AVG(CAST(score AS REAL)), 0.0) as average_score
             FROM scans",
        )
        .fetch_one(&self.pool)
        .await?;

        let total_domains: i64 = row.get("total_domains");
        let total_scans: i64 = row.get("total_scans");
        let average_score: f64 = row.get("average_score");

        // Grade distribution from latest scans only
        let grade_rows = sqlx::query(
            "SELECT grade, COUNT(*) as cnt
             FROM scans s
             INNER JOIN (
                 SELECT domain, MAX(scanned_at) as max_at FROM scans GROUP BY domain
             ) latest ON s.domain = latest.domain AND s.scanned_at = latest.max_at
             GROUP BY grade",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut dist = GradeDistribution {
            a_plus: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            f: 0,
        };

        for r in grade_rows {
            let grade: String = r.get("grade");
            let cnt: i64 = r.get("cnt");
            match grade.as_str() {
                "A+" => dist.a_plus = cnt,
                "A" => dist.a = cnt,
                "B" => dist.b = cnt,
                "C" => dist.c = cnt,
                "D" => dist.d = cnt,
                "F" => dist.f = cnt,
                _ => {}
            }
        }

        Ok(AggregateStats {
            total_domains,
            total_scans,
            average_score,
            grade_distribution: dist,
        })
    }
}

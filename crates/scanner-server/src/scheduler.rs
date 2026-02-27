use crate::storage::Storage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Spawn a background task that periodically checks for domains due for re-scanning.
pub fn spawn_scheduler<S: Storage>(storage: Arc<S>) {
    let running = Arc::new(AtomicBool::new(false));

    tokio::spawn(async move {
        let interval = Duration::from_secs(5 * 60); // Check every 5 minutes
        loop {
            tokio::time::sleep(interval).await;

            // Skip if previous cycle is still running
            if running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                tracing::debug!("Scheduler: previous cycle still running, skipping");
                continue;
            }

            run_cycle(&*storage).await;
            running.store(false, Ordering::SeqCst);
        }
    });
}

async fn run_cycle<S: Storage + ?Sized>(storage: &S) {
    let domains = match storage.get_due_domains().await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Scheduler: failed to get due domains: {e:#}");
            return;
        }
    };

    if domains.is_empty() {
        tracing::debug!("Scheduler: no domains due for scanning");
        return;
    }

    tracing::info!("Scheduler: {} domain(s) due for scanning", domains.len());

    for domain in &domains {
        tracing::info!("Scheduler: scanning {domain}");

        match scanner_engine::run_scan(domain).await {
            Ok(result) => {
                let scan_json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(e) => {
                        tracing::error!(
                            "Scheduler: failed to serialize result for {domain}: {e:#}"
                        );
                        continue;
                    }
                };
                let grade_str = result.grade.to_string();

                if let Err(e) = storage
                    .store_scan(domain, result.total_score, &grade_str, &scan_json)
                    .await
                {
                    tracing::error!("Scheduler: failed to store scan for {domain}: {e:#}");
                    continue;
                }

                if let Err(e) = storage.mark_scanned(domain).await {
                    tracing::error!(
                        "Scheduler: failed to mark {domain} as scanned: {e:#}"
                    );
                }

                tracing::info!(
                    "Scheduler: {domain} scored {}/100 ({})",
                    result.total_score,
                    result.grade
                );
            }
            Err(e) => {
                tracing::error!("Scheduler: scan failed for {domain}: {e:#}");
            }
        }

        // Brief pause between scans to be respectful
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

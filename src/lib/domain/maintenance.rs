//! Background database maintenance task.

use crate::data::DatabasePool;
use crate::service;
use std::time::Duration;
use tokio::runtime::Handle;

/// Async background task that performs rountine database tasks.
///
/// This task deletes expired jobs periodically.
pub struct Maintenance;

impl Maintenance {
    /// Spawn the database maintenance task.
    pub fn spawn(pool: DatabasePool, handle: Handle) -> Self {
        handle.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            let res = service::action::download_graph_jobs(&pool).await;
            println!("Downloading All Jobs: {:?}", res);
            loop {
                interval.tick().await;
                if let Err(e) = service::action::delete_expired(&pool).await {
                    eprintln!("failed to delete expired jobs: {}", e);
                }
                let new_jobs = service::action::fetch_and_insert_new_jobs(&pool).await;
                println!("Downloaded New Jobs:: {:?}", new_jobs);
            }
        });
        Self
    }
}

//! Background thread that commits responses to the database.

use crate::data::DatabasePool;
use crate::service::{self, ServiceError};
use crate::ShortCode;
use crossbeam_channel::TryRecvError;
use crossbeam_channel::{unbounded, Sender};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;

/// Thread-safe shared storage of pending responses.
type HitStore = Arc<Mutex<HashMap<ShortCode, u32>>>;

/// The possible errors that can occur when processing responses.
#[derive(Debug, thiserror::Error)]
enum HitCountError {
    /// Problem with the service.
    #[error("service error: {0}")]
    Service(#[from] ServiceError),
    /// Problem with the channel.
    #[error("communication error: {0}")]
    Channel(#[from] crossbeam_channel::SendError<HitCountMsg>),
}

/// Message used on the communication channel.
enum HitCountMsg {
    /// Save the responses to the database.
    Commit,
    /// Add some responses to this [`ShortCode`](crate::domain::job::field::ShortCode).
    Hit(ShortCode, u32),
}

/// A threaded hit counter.
///
/// The hit counter spawns a separate thread which manages a buffer of accumulated responses.
/// Periodically, the thread will commit the responses to the database.
///
/// This is done as a performance optimization for SQLite, since writes to a SQLite
/// database block all reads.
pub struct ResponseCounter {
    tx: Sender<HitCountMsg>,
}

impl ResponseCounter {
    /// Save the pending responses to the database.
    fn commit_responses(
        responses: HitStore,
        handle: Handle,
        pool: DatabasePool,
    ) -> Result<(), HitCountError> {
        let responses = Arc::clone(&responses);
        let responses: Vec<(ShortCode, u32)> = {
            let mut responses = responses.lock();
            let responses_vec = responses.iter().map(|(k, v)| (k.clone(), *v)).collect();
            responses.clear();
            responses_vec
        };
        handle.block_on(async move {
            let transaction = service::action::begin_transaction(&pool).await?;
            for (shortcode, responses) in responses {
                if let Err(e) =
                    service::action::increase_hit_count(&shortcode, responses, &pool).await
                {
                    eprintln!("error increasing hit count: {}", e);
                }
            }
            Ok(service::action::end_transaction(transaction).await?)
        })
    }

    /// Process an incoming [`message`](HitCountMsg).
    fn process_msg(
        msg: HitCountMsg,
        responses: HitStore,
        handle: Handle,
        pool: DatabasePool,
    ) -> Result<(), HitCountError> {
        match msg {
            HitCountMsg::Commit => Self::commit_responses(responses, handle, pool)?,
            HitCountMsg::Hit(shortcode, count) => {
                let mut hitcount = responses.lock();
                let hitcount = hitcount.entry(shortcode).or_insert(0);
                *hitcount += count;
            }
        }
        Ok(())
    }

    /// Create a new [`ResponseCounter`].
    pub fn new(pool: DatabasePool, handle: Handle) -> Self {
        let (tx, rx) = unbounded();
        let tx_clone = tx.clone();

        let _ = std::thread::spawn(move || {
            println!("HitCounter thread spawned");
            let store: HitStore = Arc::new(Mutex::new(HashMap::new()));

            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        if let Err(e) =
                            Self::process_msg(msg, store.clone(), handle.clone(), pool.clone())
                        {
                            eprintln!("message processing error: {}", e);
                        }
                    }
                    Err(e) => match e {
                        TryRecvError::Empty => {
                            std::thread::sleep(Duration::from_secs(5));
                            if let Err(e) = tx_clone.send(HitCountMsg::Commit) {
                                eprintln!("error sending commit msg to responses channel: {}", e);
                            }
                        }
                        _ => break,
                    },
                }
            }
        });

        Self { tx }
    }

    /// Add `count` number of responses to the [`Job`](crate::Job) that is referenced by the [`ShortCode`](crate::domain::job::field::ShortCode).
    pub fn hit(&self, shortcode: ShortCode, count: u32) {
        if let Err(e) = self.tx.send(HitCountMsg::Hit(shortcode, count)) {
            eprintln!("hit count error: {}", e);
        }
    }
}

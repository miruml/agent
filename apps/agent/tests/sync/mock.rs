// standard crates
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// internal crates
use config_agent::sync::{
    errors::SyncErr,
    syncer::SyncerExt,
};
use config_agent::utils::is_in_cooldown;

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use tracing::info;

type SyncFn = Box<dyn Fn() -> Result<(), SyncErr> + Send + Sync>;

pub struct MockSyncer {
    pub last_sync_attempted_at: Arc<Mutex<DateTime<Utc>>>,
    pub sync_fn: Arc<Mutex<SyncFn>>,
    pub num_sync_calls: AtomicUsize,
}

impl Default for MockSyncer {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSyncer {
    pub fn new() -> Self {
        Self {
            sync_fn: Arc::new(Mutex::new(Box::new(|| Ok(())))),
            last_sync_attempted_at: Arc::new(Mutex::new(DateTime::<Utc>::UNIX_EPOCH)),
            num_sync_calls: AtomicUsize::new(0),
        }
    }

    pub fn set_sync<F>(&self, sync_fn: F)
    where
        F: Fn() -> Result<(), SyncErr> + Send + Sync + 'static,
    {
        *self.sync_fn.lock().unwrap() = Box::new(sync_fn);
    }

    pub fn set_last_sync_attempted_at(&self, last_sync_attempted_at: DateTime<Utc>) {
        *self.last_sync_attempted_at.lock().unwrap() = last_sync_attempted_at;
    }

    pub fn num_sync_calls(&self) -> usize {
        self.num_sync_calls.load(Ordering::Relaxed)
    }
}

impl SyncerExt for MockSyncer {
    async fn shutdown(&self) -> Result<(), SyncErr> {
        Ok(())
    }

    async fn sync(&self) -> Result<(), SyncErr> {
        *self.last_sync_attempted_at.lock().unwrap() = Utc::now();
        self.num_sync_calls.fetch_add(1, Ordering::Relaxed);
        (*self.sync_fn.lock().unwrap())()
    }

    async fn get_last_sync_attempted_at(&self) -> Result<DateTime<Utc>, SyncErr> {
        Ok(*self.last_sync_attempted_at.lock().unwrap())
    }

    async fn is_in_cooldown(&self, cooldown: TimeDelta) -> Result<bool, SyncErr> {
        let last_sync_attempted_at = *self.last_sync_attempted_at.lock().unwrap();
        Ok(is_in_cooldown(last_sync_attempted_at, cooldown))
    }
}
// standard crates
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// internal crates
use config_agent::sync::{
    errors::SyncErr,
    syncer::{SyncerExt, SyncState, SyncEvent},
};

// external crates
use chrono::{DateTime, Utc};
use tokio::sync::watch;

type GetSyncStateFn = Box<dyn Fn() -> SyncState + Send + Sync>;
type IsInCooldownFn = Box<dyn Fn() -> bool + Send + Sync>;
type CooldownEndsAtFn = Box<dyn Fn() -> DateTime<Utc> + Send + Sync>;
type SyncFn = Box<dyn Fn() -> Result<(), SyncErr> + Send + Sync>;
type SyncIfNotInCooldownFn = Box<dyn Fn() -> Result<(), SyncErr> + Send + Sync>;
type SubscribeFn = Box<dyn Fn() -> watch::Receiver<SyncEvent> + Send + Sync>;


pub struct MockSyncer {
    pub last_sync_attempted_at: Arc<Mutex<DateTime<Utc>>>,
    pub num_sync_calls: AtomicUsize,
    pub get_sync_state_fn: Arc<Mutex<GetSyncStateFn>>,
    pub is_in_cooldown_fn: Arc<Mutex<IsInCooldownFn>>,
    pub cooldown_ends_at_fn: Arc<Mutex<CooldownEndsAtFn>>,
    pub sync_fn: Arc<Mutex<SyncFn>>,
    pub sync_if_not_in_cooldown_fn: Arc<Mutex<SyncIfNotInCooldownFn>>,
    pub subscribe_fn: Arc<Mutex<SubscribeFn>>,
}

impl Default for MockSyncer {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSyncer {
    pub fn new() -> Self {
        Self {
            last_sync_attempted_at: Arc::new(Mutex::new(DateTime::<Utc>::UNIX_EPOCH)),
            num_sync_calls: AtomicUsize::new(0),
            get_sync_state_fn: Arc::new(Mutex::new(Box::new(|| {
                SyncState {
                    last_sync_attempted_at: DateTime::<Utc>::UNIX_EPOCH,
                    last_successful_sync_at: DateTime::<Utc>::UNIX_EPOCH,
                    cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
                    err_streak: 0,
                }
            }))),
            is_in_cooldown_fn: Arc::new(Mutex::new(Box::new(|| false))),
            cooldown_ends_at_fn: Arc::new(Mutex::new(Box::new(Utc::now))),
            sync_if_not_in_cooldown_fn: Arc::new(Mutex::new(Box::new(|| Ok(())))),
            sync_fn: Arc::new(Mutex::new(Box::new(|| Ok(())))),
            subscribe_fn: Arc::new(Mutex::new(Box::new(|| {
                let (_, rx) = watch::channel(SyncEvent::SyncSuccess);
                rx
            }))),
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

    async fn get_sync_state(&self) -> Result<SyncState, SyncErr> {
        Ok((*self.get_sync_state_fn.lock().unwrap())())
    }

    async fn is_in_cooldown(&self) -> Result<bool, SyncErr> {
        Ok((*self.is_in_cooldown_fn.lock().unwrap())())
    }

    async fn get_cooldown_ends_at(&self) -> Result<DateTime<Utc>, SyncErr> {
        Ok((*self.cooldown_ends_at_fn.lock().unwrap())())
    }

    async fn sync(&self) -> Result<(), SyncErr> {
        *self.last_sync_attempted_at.lock().unwrap() = Utc::now();
        self.num_sync_calls.fetch_add(1, Ordering::Relaxed);
        (*self.sync_fn.lock().unwrap())()
    }

    async fn sync_if_not_in_cooldown(&self) -> Result<(), SyncErr> {
        *self.last_sync_attempted_at.lock().unwrap() = Utc::now();
        (*self.sync_if_not_in_cooldown_fn.lock().unwrap())()
    }

    async fn subscribe(&self) -> Result<watch::Receiver<SyncEvent>, SyncErr> {
        Ok((*self.subscribe_fn.lock().unwrap())())
    }
}
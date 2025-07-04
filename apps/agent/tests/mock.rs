// standard library
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool,Ordering};
use std::sync::Mutex;
use std::time::Duration;

// internal crates
use config_agent::errors::{Code, HTTPCode, MiruError};

#[derive(Debug)]
pub struct MockMiruError {
    network_err: bool,
}

impl MockMiruError {
    pub fn new(network_err: bool) -> Self {
        Self { network_err }
    }
}

impl MiruError for MockMiruError {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        self.network_err
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for MockMiruError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockMiruError")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State(bool);

impl State {
    pub const A: Self = State(true);
    pub const B: Self = State(false);
    
    pub fn is_sleeping(&self) -> bool {
        self.0
    }
}

impl From<bool> for State {
    fn from(b: bool) -> Self {
        State(b)
    }
}

impl From<State> for bool {
    fn from(s: State) -> Self {
        s.0
    }
}

// ================================== SLEEP ===================================== //
pub struct SleepController {
    target: Arc<AtomicBool>,
    actual: Arc<AtomicBool>,
    sleeps: Arc<Mutex<Vec<Duration>>>,
}

impl Default for SleepController {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepController {
    pub fn new() -> Self {
        Self {
            target: Arc::new(AtomicBool::new(State::A.into())),
            actual: Arc::new(AtomicBool::new(State::B.into())),
            sleeps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_target_state(&self) -> State {
        self.target.load(Ordering::Relaxed).into()
    }

    fn get_actual_state(&self) -> State {
        self.actual.load(Ordering::Relaxed).into()
    }

    pub fn is_sleeping(&self) -> bool {
        self.get_actual_state() == self.get_target_state()
    }

    pub async fn await_sleep(&self) {
        while !self.is_sleeping() {
            tokio::task::yield_now().await;
        }
    }

    pub async fn release(&self) {
        // the thread is sleeping if the target state equals the actual state. To release
        // it we just flip the target state
        self.target.store(
            !bool::from(self.get_actual_state()),
            Ordering::Relaxed,
        );
    }

    pub fn sleep_fn(&self) -> impl Fn(Duration) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        let sleeps = self.sleeps.clone();
        let target = self.target.clone();
        let actual = self.actual.clone();

        move |wait| {
            sleeps.lock().unwrap().push(wait);
            let target = target.clone();
            let actual = actual.clone();
            // the thread is sleeping if the target state equals the actual state. To
            // signal that the thread has begun its sleep, we set the actual state to
            // the target state.
            actual.store(
                target.load(Ordering::Relaxed), 
                Ordering::Relaxed,
            );

            Box::pin(async move {
                while target.load(Ordering::Relaxed) == actual.load(Ordering::Relaxed) {
                    tokio::task::yield_now().await;
                }
            })
        }
    }

    pub fn get_sleeps(&self) -> Vec<Duration> {
        self.sleeps.lock().unwrap().clone()
    }

    pub fn get_last_sleep(&self) -> Option<Duration> {
        self.sleeps.lock().unwrap().last().copied()
    }

    pub fn clear_sleeps(&self) {
        self.sleeps.lock().unwrap().clear();
    }

    pub fn num_sleeps(&self) -> usize {
        self.sleeps.lock().unwrap().len()
    }
}

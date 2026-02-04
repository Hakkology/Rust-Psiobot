use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct RateLimiter {
    last_action: Arc<Mutex<Option<DateTime<Utc>>>>,
    cooldown: Duration,
}

impl RateLimiter {
    pub fn new(cooldown_seconds: u64) -> Self {
        Self {
            last_action: Arc::new(Mutex::new(None)),
            cooldown: Duration::from_secs(cooldown_seconds),
        }
    }

    pub fn check_and_update(&self) -> Result<(), u64> {
        let mut last = self.last_action.lock().unwrap();
        let now = Utc::now();

        if let Some(last_time) = *last {
            let elapsed = now - last_time;
            let cooldown_seconds = self.cooldown.as_secs() as i64;

            if elapsed.num_seconds() < cooldown_seconds {
                return Err((cooldown_seconds - elapsed.num_seconds()) as u64);
            }
        }

        *last = Some(now);
        Ok(())
    }
}

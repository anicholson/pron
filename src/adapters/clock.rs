use crate::application::ports::clock::{Clock, Minute};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SystemClock;

impl SystemClock {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Minute {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Minute {
            min: ((secs / 60) % 60) as u32,
            hour: ((secs / 3600) % 24) as u32,
            dom: 1,
            mon: 1,
            dow: 0,
        }
    }
}
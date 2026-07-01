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

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z / 146097 } else { (z - 146096) / 146097 };
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

impl Clock for SystemClock {
    fn now(&self) -> Minute {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let days = secs.div_euclid(86400);
        let dow = ((days % 7) + 4) as i32;
        let dow = ((dow % 7) + 7) % 7;
        let (_, mon, dom) = civil_from_days(days);
        let hms = secs.rem_euclid(86400);
        Minute {
            min: ((hms / 60) % 60) as u32,
            hour: ((hms / 3600) % 24) as u32,
            dom,
            mon,
            dow: dow as u32,
        }
    }
}
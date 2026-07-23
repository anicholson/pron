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

fn day_of_week(days: i64) -> u32 {
    let dow = ((days % 7) + 4) as i32;
    (((dow % 7) + 7) % 7) as u32
}

fn time_of_day(secs: i64) -> (u32, u32) {
    let hms = secs.rem_euclid(86400);
    let hour = ((hms / 3600) % 24) as u32;
    let min = ((hms / 60) % 60) as u32;
    (hour, min)
}

impl Clock for SystemClock {
    fn now(&self) -> Minute {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let days = secs.div_euclid(86400);
        let dow = day_of_week(days);
        let (_, mon, dom) = civil_from_days(days);
        let (hour, min) = time_of_day(secs);
        Minute { min, hour, dom, mon, dow }
    }
}

#[cfg(test)]
mod tests {
    mod civil_from_days {
        mod when_called_with_a_known_reference_date {
            #[test]
            fn then_the_correct_year_month_and_day_are_returned() {
                let cases: [(i64, i64, u32, u32); 13] = [
                    (0, 1970, 1, 1),
                    (1, 1970, 1, 2),
                    (10957, 2000, 1, 1),
                    (11016, 2000, 2, 29),  // leap day: divisible by 400
                    (11017, 2000, 3, 1),
                    (19416, 2023, 2, 28),  // non-leap year: last day of Feb
                    (19417, 2023, 3, 1),
                    (19782, 2024, 2, 29),  // leap day: divisible by 4, not 100
                    (19783, 2024, 3, 1),
                    (47540, 2100, 2, 28),  // divisible by 100, not 400: NOT a leap year
                    (47541, 2100, 3, 1),
                    (24855, 2038, 1, 19),
                    (24856, 2038, 1, 20),
                ];
                for (days, expected_y, expected_m, expected_d) in cases {
                    let (y, m, d) = crate::adapters::clock::civil_from_days(days);
                    assert_eq!(
                        (y, m, d),
                        (expected_y, expected_m, expected_d),
                        "civil_from_days({days}) should be {expected_y}-{expected_m:02}-{expected_d:02}, got {y}-{m:02}-{d:02}"
                    );
                }
            }
        }
    }

    mod day_of_week {
        mod when_called_with_a_known_reference_date {
            #[test]
            fn then_the_correct_day_of_week_is_returned() {
                let cases: [(i64, u32); 6] = [
                    (0, 4),     // 1970-01-01, Thursday
                    (1, 5),     // 1970-01-02, Friday
                    (3, 0),     // 1970-01-04, Sunday
                    (4, 1),     // 1970-01-05, Monday
                    (10957, 6), // 2000-01-01, Saturday
                    (19783, 5), // 2024-03-01, Friday
                ];
                for (days, expected_dow) in cases {
                    let dow = crate::adapters::clock::day_of_week(days);
                    assert_eq!(
                        dow, expected_dow,
                        "day_of_week({days}) should be {expected_dow}, got {dow}"
                    );
                }
            }
        }
    }

    mod time_of_day {
        mod when_called_with_a_known_reference_time {
            #[test]
            fn then_the_correct_hour_and_minute_are_returned() {
                let cases: [(i64, u32, u32); 5] = [
                    (0, 0, 0),
                    (61, 0, 1),
                    (3661, 1, 1),
                    (86399, 23, 59),
                    (86400, 0, 0), // wraps to the next day's midnight
                ];
                for (secs, expected_hour, expected_min) in cases {
                    let (hour, min) = crate::adapters::clock::time_of_day(secs);
                    assert_eq!(
                        (hour, min),
                        (expected_hour, expected_min),
                        "time_of_day({secs}) should be {expected_hour:02}:{expected_min:02}, got {hour:02}:{min:02}"
                    );
                }
            }
        }
    }
}
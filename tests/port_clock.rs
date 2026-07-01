#[macro_export]
macro_rules! clock_contract {
    ($mod_name:ident, $make:expr) => {
        mod $mod_name {
            mod now {
                mod when_called {
                    #[test]
                    fn then_the_current_minute_tuple_is_returned() {
                        use pron::application::ports::clock::Clock;
                        let clock = $make;
                        let m = clock.now();
                        assert!(m.min <= 59, "minute should be in 0..=59, got {}", m.min);
                        assert!(m.hour <= 23, "hour should be in 0..=23, got {}", m.hour);
                        assert!(m.dom >= 1 && m.dom <= 31, "day-of-month should be in 1..=31, got {}", m.dom);
                        assert!(m.mon >= 1 && m.mon <= 12, "month should be in 1..=12, got {}", m.mon);
                        assert!(m.dow <= 6, "day-of-week should be in 0..=6, got {}", m.dow);
                    }
                }
            }
        }
    };
}

clock_contract!(in_memory, pron::application::ports::clock::in_memory::InMemoryClock::with(30, 14, 15, 7, 2));
clock_contract!(real, pron::adapters::clock::SystemClock::new());
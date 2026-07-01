#[macro_export]
macro_rules! clock_contract {
    ($make:expr) => {
        mod clock_contract {
            mod now {
                mod when_called {
                    #[test]
                    fn then_the_current_minute_tuple_is_returned() {
                        use pron::application::ports::clock::Clock;
                        let clock = $make;
                        let _m = clock.now();
                    }
                }
            }
        }
    };
}

clock_contract!(pron::application::ports::clock::in_memory::InMemoryClock::with(0, 0, 1, 1, 0));
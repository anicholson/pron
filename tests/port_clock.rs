use pron::application::ports::clock::in_memory::InMemoryClock;

#[macro_export]
macro_rules! clock_contract {
    ($make:expr) => {
        mod clock_contract {
            use super::*;

            mod now {
                mod when_called {
                    #[test]
                    fn then_the_current_minute_tuple_is_returned() {
                        let clock = $make;
                        let _m = clock.now();
                    }
                }
            }
        }
    };
}

clock_contract!(InMemoryClock::with(0, 0, 1, 1, 0));
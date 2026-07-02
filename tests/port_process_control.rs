#[macro_export]
macro_rules! process_control_contract {
    ($mod_name:ident, $make:expr, $expected:expr) => {
        mod $mod_name {
            mod current_pid {
                mod when_called {
                    #[test]
                    fn then_it_returns_the_expected_pid() {
                        use pron::application::ports::process_control::ProcessControl;
                        let pc = $make;
                        assert_eq!(pc.current_pid(), $expected);
                    }
                }
            }
        }
    };
}

process_control_contract!(
    in_memory,
    pron::application::ports::process_control::in_memory::InMemoryProcessControl::with_pid(4242),
    4242u32
);

process_control_contract!(
    real,
    pron::adapters::process_control::RealProcessControl::new(),
    std::process::id()
);
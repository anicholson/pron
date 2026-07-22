#[macro_export]
macro_rules! process_control_contract {
    ($mod_name:ident, $make:expr, $expected:expr $(, $live_pron_make:expr)?) => {
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
            mod is_live_pron {
                mod when_called_with_a_pid_that_is_not_a_live_pron_process {
                    #[test]
                    fn then_false_is_returned() {
                        use pron::application::ports::process_control::ProcessControl;
                        let pc = $make;
                        assert!(!pc.is_live_pron(999999));
                    }
                }
                $(
                mod when_called_with_a_configured_live_pron_pid {
                    #[test]
                    fn then_true_is_returned() {
                        use pron::application::ports::process_control::ProcessControl;
                        let pc = $live_pron_make;
                        assert!(pc.is_live_pron(4242));
                    }
                }
                )?
            }
        }
    };
}

process_control_contract!(
    in_memory,
    pron::application::ports::process_control::in_memory::InMemoryProcessControl::with_pid(4242),
    4242u32,
    pron::application::ports::process_control::in_memory::InMemoryProcessControl::with_live_pron(4242)
);

process_control_contract!(
    real,
    pron::adapters::process_control::RealProcessControl::new(),
    std::process::id()
);
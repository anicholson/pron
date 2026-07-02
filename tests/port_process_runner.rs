#[macro_export]
macro_rules! process_runner_contract {
    ($make:expr) => {
        mod process_runner_contract {
            mod run {
                mod when_called_with_a_command {
                    #[test]
                    fn then_the_command_is_executed_stdout_returned_and_exit_status_zero() {
                        use pron::application::ports::process_runner::ProcessRunner;
                        let runner = $make;
                        let result = runner.run("echo hi");
                        assert!(result.is_ok(), "run should succeed: {:?}", result.err());
                        let result = result.unwrap();
                        assert_eq!(result.exit_status, 0, "exit status should be 0");
                        assert!(
                            result.stdout.contains("hi"),
                            "stdout should contain 'hi': {}",
                            result.stdout
                        );
                    }
                }
            }
        }
    };
}

process_runner_contract!(pron::adapters::process_runner::ShProcessRunner::new());
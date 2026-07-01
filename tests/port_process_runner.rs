#[macro_export]
macro_rules! process_runner_contract {
    ($make:expr) => {
        mod process_runner_contract {
            mod run {
                mod when_called_with_a_command {
                    #[test]
                    fn then_the_command_is_executed_and_stdout_returned() {
                        use pron::application::ports::process_runner::ProcessRunner;
                        let runner = $make;
                        let result = runner.run("echo hi");
                        assert!(result.is_ok());
                        let output = result.unwrap();
                        assert!(output.contains("hi"), "output should contain 'hi': {output}");
                    }
                }
            }
        }
    };
}

process_runner_contract!(pron::adapters::process_runner::ShProcessRunner::new());
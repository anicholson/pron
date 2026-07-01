#[macro_export]
macro_rules! process_runner_contract {
    ($make:expr) => {
        mod process_runner_contract {
            use super::*;

            mod run {
                mod when_called_with_a_command {
                    #[test]
                    fn then_the_command_is_executed_and_stdout_returned() {
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

use pron::adapters::process_runner::ShProcessRunner;

process_runner_contract!(ShProcessRunner::new());
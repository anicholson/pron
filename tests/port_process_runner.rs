#[macro_export]
macro_rules! process_runner_contract {
    ($mod_name:ident, $make:expr, $command:expr) => {
        mod $mod_name {
            mod run {
                mod when_called_with_a_command {
                    #[test]
                    fn then_the_command_is_executed_and_stdout_returned() {
                        use pron::application::ports::process_runner::ProcessRunner;
                        let runner = $make;
                        let result = runner.run($command);
                        assert!(result.is_ok(), "run should succeed: {:?}", result.err());
                        let result = result.unwrap();
                        assert!(
                            result.stdout.contains("hi"),
                            "stdout should contain 'hi': {}",
                            result.stdout
                        );
                    }

                    #[test]
                    fn and_the_exit_status_is_returned_as_zero_on_success() {
                        use pron::application::ports::process_runner::ProcessRunner;
                        let runner = $make;
                        let result = runner.run($command).unwrap();
                        assert_eq!(result.exit_status, 0, "exit status should be 0");
                    }

                    #[test]
                    fn and_stderr_is_captured_separately_from_stdout() {
                        use pron::application::ports::process_runner::ProcessRunner;
                        let runner = $make;
                        let result = runner.run($command).unwrap();
                        assert!(
                            result.stderr.contains("bye"),
                            "stderr should contain 'bye': {}",
                            result.stderr
                        );
                        assert!(
                            !result.stdout.contains("bye"),
                            "stdout should NOT contain stderr content: {}",
                            result.stdout
                        );
                    }
                }
            }
        }
    };
}

process_runner_contract!(
    in_memory,
    pron::application::ports::process_runner::in_memory::InMemoryProcessRunner::with_output(
        "hi\n", "bye\n"
    ),
    "echo hi; echo bye 1>&2"
);
process_runner_contract!(
    real,
    pron::adapters::process_runner::ShProcessRunner::new(),
    "echo hi; echo bye 1>&2"
);

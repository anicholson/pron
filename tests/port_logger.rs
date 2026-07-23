#[macro_export]
macro_rules! logger_contract {
    ($mod_name:ident, $setup:expr) => {
        mod $mod_name {
            mod log_start {
                mod when_called {
                    #[test]
                    fn then_a_start_event_containing_mode_and_entry_count_is_recorded() {
                        use pron::application::ports::logger::Logger;
                        let (logger, read) = $setup;
                        logger.log_start("daemon", "/etc/crontab", 3);
                        let recorded = read();
                        assert!(
                            recorded.contains("start"),
                            "expected 'start' in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("daemon"),
                            "expected mode 'daemon' in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("3"),
                            "expected entry_count '3' in recorded output: {recorded}"
                        );
                    }
                }
            }
            mod log_job {
                mod when_called {
                    #[test]
                    fn then_begin_and_end_markers_with_command_and_output_are_recorded() {
                        use pron::application::ports::logger::Logger;
                        let (logger, read) = $setup;
                        logger.log_job("job-one", "stdout-payload", "");
                        let recorded = read();
                        assert!(
                            recorded.contains("--- begin: job-one"),
                            "expected begin marker in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("stdout-payload"),
                            "expected output in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("--- end: job-one"),
                            "expected end marker in recorded output: {recorded}"
                        );
                        assert!(
                            !recorded.contains("--- stderr ---"),
                            "no stderr section should be recorded when stderr is empty: {recorded}"
                        );
                    }

                    #[test]
                    fn and_stderr_is_included_between_the_markers_when_the_command_produced_any() {
                        use pron::application::ports::logger::Logger;
                        let (logger, read) = $setup;
                        logger.log_job("job-two", "", "stderr-payload");
                        let recorded = read();
                        assert!(
                            recorded.contains("stderr-payload"),
                            "expected stderr content in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("--- begin:") && recorded.contains("--- end:"),
                            "stderr should still be recorded between the begin/end markers: {recorded}"
                        );
                    }
                }
            }
            mod log_job_exit {
                mod when_called_with_a_command_and_a_non_zero_exit_status {
                    #[test]
                    fn then_an_exit_code_line_naming_the_command_and_code_is_recorded() {
                        use pron::application::ports::logger::Logger;
                        let (logger, read) = $setup;
                        logger.log_job_exit("false", 1);
                        let recorded = read();
                        assert!(
                            recorded.contains("exited with code 1"),
                            "expected exit-code line in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("false"),
                            "expected command name in recorded output: {recorded}"
                        );
                    }
                }
            }
            mod log_spawn_failure {
                mod when_called_with_a_command_and_an_error {
                    #[test]
                    fn then_a_spawn_failure_line_naming_the_command_and_error_is_recorded() {
                        use pron::application::ports::logger::Logger;
                        let (logger, read) = $setup;
                        logger.log_spawn_failure("missing-cmd", "not found");
                        let recorded = read();
                        assert!(
                            recorded.contains("failed to spawn"),
                            "expected spawn-failure line in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("missing-cmd"),
                            "expected command name in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("not found"),
                            "expected error text in recorded output: {recorded}"
                        );
                    }
                }
            }
        }
    };
}

logger_contract!(
    in_memory,
    {
        let logger = pron::application::ports::logger::in_memory::InMemoryLogger::default();
        let read = {
            let logger = logger.clone();
            move || logger.events.lock().unwrap().join("\n")
        };
        (logger, read)
    }
);

logger_contract!(
    real,
    {
        let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let logfile = dir.path().join(".pron.log");
        let logger = pron::adapters::logger::RealLogger::new(dir.path());
        let read = move || std::fs::read_to_string(&logfile).unwrap_or_default();
        (logger, read)
    }
);
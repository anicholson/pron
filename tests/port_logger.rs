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
                        logger.log_job("echo hi", "hi");
                        let recorded = read();
                        assert!(
                            recorded.contains("--- begin: echo hi"),
                            "expected begin marker in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("hi"),
                            "expected output in recorded output: {recorded}"
                        );
                        assert!(
                            recorded.contains("--- end: echo hi"),
                            "expected end marker in recorded output: {recorded}"
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
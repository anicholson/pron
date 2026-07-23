use crate::application::ports::{filesystem::Filesystem, logger::Logger, process_control::ProcessControl};
use crate::domain::crontab;

pub struct Start<U: Filesystem, L: Logger, P: ProcessControl> {
    fs: U,
    logger: L,
    proc: P,
}

impl<U: Filesystem, L: Logger, P: ProcessControl> Start<U, L, P> {
    pub fn new(fs: U, logger: L, proc: P) -> Self {
        Self { fs, logger, proc }
    }

    pub fn execute(&self, crontab_text: &str, mode: &str) -> Result<Vec<crontab::Entry>, String> {
        let entries = crontab::parse(crontab_text).map_err(|e| e.to_string())?;
        if let Some(holder) = self.fs.read_pidfile()?
            && self.proc.is_live_pron(holder)
        {
            return Err(format!("pron is already running (pid {holder})"));
        }
        let pid = self.proc.current_pid();
        self.fs.write_pidfile(pid)?;
        self.logger.log_start(mode, ".prontab", entries.len());
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    mod execute {
        mod when_called_with_a_valid_crontab_and_daemon_mode {
            #[test]
            fn then_the_pidfile_is_written_with_the_current_pid() {
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                start.execute("* * * * * echo hi\n", "daemon").unwrap();

                assert_eq!(*fs.pid.lock().unwrap(), Some(4242));
            }

            #[test]
            fn then_a_start_event_is_logged() {
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                start.execute("* * * * * echo hi\n", "daemon").unwrap();

                let events = logger.events.lock().unwrap();
                assert_eq!(events.len(), 1);
                assert!(events[0].contains("start"), "event: {}", events[0]);
                assert!(events[0].contains("daemon"), "event: {}", events[0]);
                assert!(events[0].contains("entries=1"), "event: {}", events[0]);
            }
        }

        mod when_called_while_a_live_pron_process_holds_the_pidfile {
            #[test]
            fn then_an_error_naming_the_holding_pid_is_returned() {
                use crate::application::ports::filesystem::Filesystem;
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                fs.write_pidfile(4242).unwrap();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_live_pron(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                let result = start.execute("* * * * * echo hi\n", "daemon");
                let err = result
                    .expect_err("start should be refused while a live pron holds the pidfile");
                assert!(
                    err.contains("4242"),
                    "error should name the holding pid, got: {err}"
                );
            }

            #[test]
            fn and_the_pidfile_is_unchanged() {
                use crate::application::ports::filesystem::Filesystem;
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                fs.write_pidfile(4242).unwrap();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_live_pron(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                let _ = start.execute("* * * * * echo hi\n", "daemon");

                assert_eq!(
                    *fs.pid.lock().unwrap(),
                    Some(4242),
                    "the pidfile should be unchanged after a refused start"
                );
            }

            #[test]
            fn and_no_start_event_is_logged() {
                use crate::application::ports::filesystem::Filesystem;
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                fs.write_pidfile(4242).unwrap();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_live_pron(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                let _ = start.execute("* * * * * echo hi\n", "daemon");

                assert!(
                    logger.events.lock().unwrap().is_empty(),
                    "no start event should be logged after a refused start"
                );
            }
        }

        mod when_called_while_the_pidfile_names_a_stale_pid {
            #[test]
            fn then_the_pidfile_is_replaced_with_the_current_pid() {
                use crate::application::ports::filesystem::Filesystem;
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                fs.write_pidfile(4242).unwrap();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(5555);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                start.execute("* * * * * echo hi\n", "daemon").unwrap();

                assert_eq!(
                    *fs.pid.lock().unwrap(),
                    Some(5555),
                    "the stale pidfile should be replaced with the current pid"
                );
            }

            #[test]
            fn then_a_start_event_is_logged() {
                use crate::application::ports::filesystem::Filesystem;
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                fs.write_pidfile(4242).unwrap();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(5555);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                start.execute("* * * * * echo hi\n", "daemon").unwrap();

                let events = logger.events.lock().unwrap();
                assert_eq!(events.len(), 1);
                assert!(events[0].contains("start"), "event: {}", events[0]);
            }
        }

        mod if_the_pidfile_cannot_be_parsed {
            #[test]
            fn then_an_error_is_returned_and_the_pidfile_is_unchanged() {
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::with_read_error("corrupt pidfile");
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(5555);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                let result = start.execute("* * * * * echo hi\n", "daemon");

                assert!(
                    result.is_err(),
                    "start should fail when the pidfile cannot be parsed"
                );
                assert_eq!(
                    *fs.pid.lock().unwrap(),
                    None,
                    "the pidfile should be unchanged (no write) after the read error"
                );
                assert!(
                    logger.events.lock().unwrap().is_empty(),
                    "no start event should be logged after the read error"
                );
            }
        }

        mod if_the_pidfile_cannot_be_written {
            #[test]
            fn then_an_error_is_returned_and_no_start_event_is_logged() {
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::with_write_error("disk full");
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                let result = start.execute("* * * * * echo hi\n", "daemon");

                assert!(
                    result.is_err(),
                    "start should fail when the pidfile cannot be written"
                );
                assert!(
                    logger.events.lock().unwrap().is_empty(),
                    "no start event should be logged after a write failure"
                );
            }
        }

        mod if_called_with_an_invalid_crontab {
            #[test]
            fn then_a_parse_error_is_returned_without_writing_the_pidfile() {
                use crate::application::ports::filesystem::in_memory::InMemoryFilesystem;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_control::in_memory::InMemoryProcessControl;
                use crate::application::start::Start;

                let fs = InMemoryFilesystem::default();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                let result = start.execute("not * * * * echo hi\n", "daemon");
                assert!(result.is_err());
                assert_eq!(*fs.pid.lock().unwrap(), None);
                assert!(logger.events.lock().unwrap().is_empty());
            }
        }
    }
}
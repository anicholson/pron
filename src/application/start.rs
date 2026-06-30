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

    pub fn execute(&self, crontab_text: &str, mode: &str) -> Result<(), String> {
        let entries = crontab::parse(crontab_text).map_err(|e| e.to_string())?;
        let pid = self.proc.current_pid();
        self.fs.write_pidfile(pid)?;
        self.logger.log_start(mode, ".prontab", entries.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{
        filesystem::in_memory::InMemoryFilesystem,
        logger::in_memory::InMemoryLogger,
        process_control::in_memory::InMemoryProcessControl,
    };

    mod execute {
        mod when_called_with_a_valid_crontab_and_daemon_mode {
            #[test]
            fn then_the_pidfile_is_written_with_the_current_pid() {
                let fs = InMemoryFilesystem::default();
                let logger = InMemoryLogger::default();
                let proc = InMemoryProcessControl::with_pid(4242);
                let start = Start::new(fs.clone(), logger.clone(), proc);

                start.execute("* * * * * echo hi\n", "daemon").unwrap();

                assert_eq!(*fs.pid.lock().unwrap(), Some(4242));
            }

            #[test]
            fn then_a_start_event_is_logged() {
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
    }
}

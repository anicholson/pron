pub trait Logger: Send + Sync {
    fn log_start(&self, mode: &str, crontab_path: &str, entry_count: usize);
    fn log_job(&self, command: &str, output: &str);
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::Logger;
    use std::sync::{Arc, Mutex};

    #[derive(Default, Clone)]
    pub struct InMemoryLogger {
        pub events: Arc<Mutex<Vec<String>>>,
    }

    impl Logger for InMemoryLogger {
        fn log_start(&self, mode: &str, crontab_path: &str, entry_count: usize) {
            self.events.lock().unwrap().push(format!(
                "start mode={} crontab={} entries={}",
                mode, crontab_path, entry_count
            ));
        }

        fn log_job(&self, command: &str, output: &str) {
            self.events.lock().unwrap().push(format!(
                "--- begin: {} ---\n{}\n--- end: {} ---",
                command, output, command
            ));
        }
    }
}
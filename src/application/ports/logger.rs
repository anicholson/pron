pub trait Logger: Send + Sync {
    fn log_start(&self, mode: &str, crontab_path: &str, entry_count: usize);
    fn log_job(&self, command: &str, stdout: &str, stderr: &str);
    fn log_job_exit(&self, command: &str, exit_status: i32);
    fn log_spawn_failure(&self, command: &str, error: &str);
}

impl<L: Logger + ?Sized> Logger for Box<L> {
    fn log_start(&self, mode: &str, crontab_path: &str, entry_count: usize) {
        (**self).log_start(mode, crontab_path, entry_count);
    }
    fn log_job(&self, command: &str, stdout: &str, stderr: &str) {
        (**self).log_job(command, stdout, stderr);
    }
    fn log_job_exit(&self, command: &str, exit_status: i32) {
        (**self).log_job_exit(command, exit_status);
    }
    fn log_spawn_failure(&self, command: &str, error: &str) {
        (**self).log_spawn_failure(command, error);
    }
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

        fn log_job(&self, command: &str, stdout: &str, stderr: &str) {
            let mut msg = format!("--- begin: {} ---\n{}", command, stdout);
            if !stderr.is_empty() {
                msg.push_str(&format!("\n--- stderr ---\n{}", stderr));
            }
            msg.push_str(&format!("\n--- end: {} ---", command));
            self.events.lock().unwrap().push(msg);
        }

        fn log_job_exit(&self, command: &str, exit_status: i32) {
            self.events.lock().unwrap().push(format!(
                "job exited with code {}: {}",
                exit_status, command
            ));
        }

        fn log_spawn_failure(&self, command: &str, error: &str) {
            self.events.lock().unwrap().push(format!(
                "failed to spawn: {}: {}",
                command, error
            ));
        }
    }
}
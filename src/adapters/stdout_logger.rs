use crate::application::ports::logger::Logger;
use std::io::Write;

pub struct StdoutLogger;

impl StdoutLogger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdoutLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger for StdoutLogger {
    fn log_start(&self, mode: &str, crontab_path: &str, entry_count: usize) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(
            out,
            "start mode={} crontab={} entries={}",
            mode, crontab_path, entry_count
        );
    }

    fn log_job(&self, _command: &str, output: &str) {
        let mut out = std::io::stdout().lock();
        if !output.is_empty() {
            let _ = write!(out, "{}", output);
            if !output.ends_with('\n') {
                let _ = writeln!(out);
            }
        }
    }

    fn log_job_exit(&self, command: &str, exit_status: i32) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "job exited with code {}: {}", exit_status, command);
    }

    fn log_spawn_failure(&self, command: &str, error: &str) {
        let mut out = std::io::stderr().lock();
        let _ = writeln!(out, "failed to spawn: {}: {}", command, error);
    }
}

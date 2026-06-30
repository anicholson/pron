use crate::application::ports::logger::Logger;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub struct RealLogger<'a> {
    log_path: std::path::PathBuf,
    _marker: std::marker::PhantomData<&'a Path>,
}

impl<'a> RealLogger<'a> {
    pub fn new(dir: &'a Path) -> Self {
        Self {
            log_path: dir.join(".pron.log"),
            _marker: std::marker::PhantomData,
        }
    }

    fn append(&self, line: &str) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log_path) {
            let _ = writeln!(file, "{line}");
        }
    }
}

impl<'a> Logger for RealLogger<'a> {
    fn log_start(&self, mode: &str, crontab_path: &str, entry_count: usize) {
        self.append(&format!(
            "start mode={} crontab={} entries={}",
            mode, crontab_path, entry_count
        ));
    }

    fn log_job(&self, command: &str, output: &str) {
        self.append(&format!("--- begin: {} ---", command));
        if !output.is_empty() {
            self.append(output.trim_end());
        }
        self.append(&format!("--- end: {} ---", command));
    }
}

use crate::application::ports::process_runner::ProcessRunner;
use std::process::Command;

pub struct ShProcessRunner;

impl ShProcessRunner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShProcessRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRunner for ShProcessRunner {
    fn run(&self, command: &str) -> Result<String, String> {
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| format!("spawn: {e}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}
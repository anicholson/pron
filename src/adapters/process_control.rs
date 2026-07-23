use crate::application::ports::process_control::ProcessControl;
use std::path::Path;

pub struct RealProcessControl;

impl RealProcessControl {
    pub fn new() -> Self {
        Self
    }

    pub fn process_is_alive(pid: i32) -> bool {
        unsafe { libc::kill(pid, 0) == 0 }
    }

    #[cfg(target_os = "linux")]
    pub fn looks_like_pron(pid: i32) -> bool {
        let Ok(cmdline) = std::fs::read(format!("/proc/{pid}/cmdline")) else {
            return false;
        };
        let first = cmdline
            .split(|&b| b == 0)
            .next()
            .unwrap_or(&[]);
        let basename = Path::new(std::str::from_utf8(first).unwrap_or(""))
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        basename.contains("pron")
    }

    #[cfg(not(target_os = "linux"))]
    pub fn looks_like_pron(_pid: i32) -> bool {
        true
    }
}

impl Default for RealProcessControl {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessControl for RealProcessControl {
    fn current_pid(&self) -> u32 {
        std::process::id()
    }

    fn is_live_pron(&self, pid: u32) -> bool {
        Self::process_is_alive(pid as i32) && Self::looks_like_pron(pid as i32)
    }
}

#[cfg(test)]
mod tests {
    mod process_is_alive {
        mod when_called_with_the_current_process_id {
            #[test]
            fn then_true_is_returned() {
                let pid = std::process::id() as i32;
                assert!(crate::adapters::process_control::RealProcessControl::process_is_alive(pid));
            }
        }

        mod when_called_with_a_pid_that_does_not_exist {
            #[test]
            fn then_false_is_returned() {
                assert!(!crate::adapters::process_control::RealProcessControl::process_is_alive(999999));
            }
        }
    }
}

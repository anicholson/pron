use crate::application::ports::process_control::ProcessControl;

pub struct RealProcessControl;

impl RealProcessControl {
    pub fn new() -> Self {
        Self
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
}

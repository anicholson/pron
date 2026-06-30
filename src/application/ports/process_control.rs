pub trait ProcessControl: Send + Sync {
    fn current_pid(&self) -> u32;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::ProcessControl;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Default)]
    pub struct InMemoryProcessControl {
        pid: AtomicU32,
    }

    impl InMemoryProcessControl {
        pub fn with_pid(pid: u32) -> Self {
            Self { pid: AtomicU32::new(pid) }
        }
    }

    impl ProcessControl for InMemoryProcessControl {
        fn current_pid(&self) -> u32 {
            self.pid.load(Ordering::Relaxed)
        }
    }
}

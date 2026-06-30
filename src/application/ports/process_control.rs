pub trait ProcessControl: Send + Sync {
    fn current_pid(&self) -> u32;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::ProcessControl;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Clone)]
    pub struct InMemoryProcessControl {
        pid: Arc<AtomicU32>,
    }

    impl Default for InMemoryProcessControl {
        fn default() -> Self {
            Self { pid: Arc::new(AtomicU32::new(0)) }
        }
    }

    impl InMemoryProcessControl {
        pub fn with_pid(pid: u32) -> Self {
            Self { pid: Arc::new(AtomicU32::new(pid)) }
        }
    }

    impl ProcessControl for InMemoryProcessControl {
        fn current_pid(&self) -> u32 {
            self.pid.load(Ordering::Relaxed)
        }
    }
}

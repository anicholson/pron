pub trait ProcessControl: Send + Sync {
    fn current_pid(&self) -> u32;
    fn is_live_pron(&self, pid: u32) -> bool;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::ProcessControl;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct InMemoryProcessControl {
        pid: Arc<AtomicU32>,
        live_prons: Arc<Mutex<Vec<u32>>>,
    }

    impl Default for InMemoryProcessControl {
        fn default() -> Self {
            Self {
                pid: Arc::new(AtomicU32::new(0)),
                live_prons: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl InMemoryProcessControl {
        pub fn with_pid(pid: u32) -> Self {
            Self {
                pid: Arc::new(AtomicU32::new(pid)),
                live_prons: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn with_live_pron(pid: u32) -> Self {
            Self {
                pid: Arc::new(AtomicU32::new(0)),
                live_prons: Arc::new(Mutex::new(vec![pid])),
            }
        }
    }

    impl ProcessControl for InMemoryProcessControl {
        fn current_pid(&self) -> u32 {
            self.pid.load(Ordering::Relaxed)
        }

        fn is_live_pron(&self, pid: u32) -> bool {
            self.live_prons.lock().unwrap().contains(&pid)
        }
    }
}

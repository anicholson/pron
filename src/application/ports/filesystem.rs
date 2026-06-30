pub trait Filesystem: Send + Sync {
    fn write_pidfile(&self, pid: u32) -> Result<(), String>;
    fn remove_pidfile(&self) -> Result<(), String>;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::Filesystem;
    use std::sync::{Arc, Mutex};

    #[derive(Default, Clone)]
    pub struct InMemoryFilesystem {
        pub pid: Arc<Mutex<Option<u32>>>,
    }

    impl Filesystem for InMemoryFilesystem {
        fn write_pidfile(&self, pid: u32) -> Result<(), String> {
            *self.pid.lock().unwrap() = Some(pid);
            Ok(())
        }

        fn remove_pidfile(&self) -> Result<(), String> {
            *self.pid.lock().unwrap() = None;
            Ok(())
        }
    }
}

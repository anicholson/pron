pub trait Filesystem: Send + Sync {
    fn read_pidfile(&self) -> Result<Option<u32>, String>;
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
        read_error: Option<String>,
        write_error: Option<String>,
    }

    impl InMemoryFilesystem {
        pub fn with_read_error(error: &str) -> Self {
            Self {
                pid: Arc::new(Mutex::new(None)),
                read_error: Some(error.to_string()),
                write_error: None,
            }
        }

        pub fn with_write_error(error: &str) -> Self {
            Self {
                pid: Arc::new(Mutex::new(None)),
                read_error: None,
                write_error: Some(error.to_string()),
            }
        }
    }

    impl Filesystem for InMemoryFilesystem {
        fn read_pidfile(&self) -> Result<Option<u32>, String> {
            if let Some(e) = &self.read_error {
                return Err(e.clone());
            }
            Ok(*self.pid.lock().unwrap())
        }

        fn write_pidfile(&self, pid: u32) -> Result<(), String> {
            if let Some(e) = &self.write_error {
                return Err(e.clone());
            }
            *self.pid.lock().unwrap() = Some(pid);
            Ok(())
        }

        fn remove_pidfile(&self) -> Result<(), String> {
            *self.pid.lock().unwrap() = None;
            Ok(())
        }
    }
}

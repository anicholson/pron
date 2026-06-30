pub trait ProcessRunner: Send + Sync {
    fn run(&self, command: &str) -> Result<String, String>;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::ProcessRunner;
    use std::sync::{Arc, Mutex};

    #[derive(Default, Clone)]
    pub struct InMemoryProcessRunner {
        pub commands: Arc<Mutex<Vec<String>>>,
    }

    impl ProcessRunner for InMemoryProcessRunner {
        fn run(&self, command: &str) -> Result<String, String> {
            self.commands.lock().unwrap().push(command.to_string());
            Ok(String::new())
        }
    }
}
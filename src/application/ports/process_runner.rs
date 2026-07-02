pub struct RunResult {
    pub exit_status: i32,
    pub stdout: String,
}

pub trait ProcessRunner: Send + Sync {
    fn run(&self, command: &str) -> Result<RunResult, String>;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::{ProcessRunner, RunResult};
    use std::sync::{Arc, Mutex};

    #[derive(Default, Clone)]
    pub struct InMemoryProcessRunner {
        pub commands: Arc<Mutex<Vec<String>>>,
        pub spawn_error: Arc<Mutex<Option<String>>>,
        pub exit_status: Arc<Mutex<i32>>,
    }

    impl InMemoryProcessRunner {
        pub fn failing_spawn(message: &str) -> Self {
            Self {
                spawn_error: Arc::new(Mutex::new(Some(message.to_string()))),
                ..Default::default()
            }
        }

        pub fn with_exit_status(code: i32) -> Self {
            Self {
                exit_status: Arc::new(Mutex::new(code)),
                ..Default::default()
            }
        }
    }

    impl ProcessRunner for InMemoryProcessRunner {
        fn run(&self, command: &str) -> Result<RunResult, String> {
            self.commands.lock().unwrap().push(command.to_string());
            if let Some(e) = self.spawn_error.lock().unwrap().take() {
                return Err(e);
            }
            let exit_status = *self.exit_status.lock().unwrap();
            Ok(RunResult { exit_status, stdout: String::new() })
        }
    }
}
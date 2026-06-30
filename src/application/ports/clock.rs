pub struct Minute {
    pub min: u32,
    pub hour: u32,
    pub dom: u32,
    pub mon: u32,
    pub dow: u32,
}

pub trait Clock: Send + Sync {
    fn now(&self) -> Minute;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use super::{Clock, Minute};

    #[derive(Clone)]
    pub struct InMemoryClock {
        minute: std::sync::Arc<std::sync::Mutex<Minute>>,
    }

    impl InMemoryClock {
        pub fn with(min: u32, hour: u32, dom: u32, mon: u32, dow: u32) -> Self {
            Self {
                minute: std::sync::Arc::new(std::sync::Mutex::new(Minute {
                    min, hour, dom, mon, dow,
                })),
            }
        }

        pub fn set(&self, min: u32, hour: u32, dom: u32, mon: u32, dow: u32) {
            *self.minute.lock().unwrap() = Minute { min, hour, dom, mon, dow };
        }
    }

    impl Clock for InMemoryClock {
        fn now(&self) -> Minute {
            let m = self.minute.lock().unwrap();
            Minute {
                min: m.min,
                hour: m.hour,
                dom: m.dom,
                mon: m.mon,
                dow: m.dow,
            }
        }
    }
}
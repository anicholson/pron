#[macro_export]
macro_rules! filesystem_contract {
    ($mod_name:ident, $setup:expr) => {
        mod $mod_name {
            mod write_pidfile {
                mod when_called_with_a_pid {
                    #[test]
                    fn then_the_pid_is_retrievable() {
                        use pron::application::ports::filesystem::Filesystem;
                        let (fs, read) = $setup;
                        fs.write_pidfile(1234).unwrap();
                        assert_eq!(read(&fs), Some(1234u32));
                    }
                }
            }
            mod remove_pidfile {
                mod when_called_after_a_write {
                    #[test]
                    fn then_the_pid_is_no_longer_retrievable() {
                        use pron::application::ports::filesystem::Filesystem;
                        let (fs, read) = $setup;
                        fs.write_pidfile(1234).unwrap();
                        fs.remove_pidfile().unwrap();
                        assert_eq!(read(&fs), None);
                    }
                }
            }
        }
    };
}

filesystem_contract!(
    in_memory,
    {
        let fs = pron::application::ports::filesystem::in_memory::InMemoryFilesystem::default();
        let read = |fs: &pron::application::ports::filesystem::in_memory::InMemoryFilesystem| {
            fs.pid.lock().unwrap().clone()
        };
        (fs, read)
    }
);

filesystem_contract!(
    real,
    {
        let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let pidfile = dir.path().join(".pron.pid");
        let fs = pron::adapters::fs::RealFilesystem::new(dir.path());
        let read = move |_fs: &pron::adapters::fs::RealFilesystem| {
            std::fs::read_to_string(&pidfile)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
        };
        (fs, read)
    }
);
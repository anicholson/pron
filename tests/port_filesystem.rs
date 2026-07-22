#[macro_export]
macro_rules! filesystem_contract {
    ($mod_name:ident, $setup:expr, $corrupt_setup:expr) => {
        mod $mod_name {
            mod read_pidfile {
                mod when_no_pidfile_has_been_written {
                    #[test]
                    fn then_none_is_returned() {
                        use pron::application::ports::filesystem::Filesystem;
                        let (fs, _read) = $setup;
                        assert_eq!(fs.read_pidfile().unwrap(), None);
                    }
                }
                mod when_called_after_a_write {
                    #[test]
                    fn then_the_written_pid_is_returned() {
                        use pron::application::ports::filesystem::Filesystem;
                        let (fs, _read) = $setup;
                        fs.write_pidfile(1234).unwrap();
                        assert_eq!(fs.read_pidfile().unwrap(), Some(1234u32));
                    }
                }
                mod if_the_pidfile_content_is_invalid {
                    #[test]
                    fn then_an_error_is_returned() {
                        use pron::application::ports::filesystem::Filesystem;
                        let fs = $corrupt_setup;
                        assert!(fs.read_pidfile().is_err());
                    }
                }
            }
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
    },
    {
        pron::application::ports::filesystem::in_memory::InMemoryFilesystem::with_read_error(
            "corrupt pidfile",
        )
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
    },
    {
        let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        std::fs::write(dir.path().join(".pron.pid"), "not-a-pid\n").unwrap();
        pron::adapters::fs::RealFilesystem::new(dir.path())
    }
);
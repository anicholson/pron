#[macro_export]
macro_rules! process_control_contract {
    ($mod_name:ident, $make:expr, $expected:expr $(, $live_pron_check:expr)?) => {
        mod $mod_name {
            mod current_pid {
                mod when_called {
                    #[test]
                    fn then_it_returns_the_expected_pid() {
                        use pron::application::ports::process_control::ProcessControl;
                        let pc = $make;
                        assert_eq!(pc.current_pid(), $expected);
                    }
                }
            }
            mod is_live_pron {
                mod when_called_with_a_pid_that_is_not_a_live_pron_process {
                    #[test]
                    fn then_false_is_returned() {
                        use pron::application::ports::process_control::ProcessControl;
                        let pc = $make;
                        assert!(!pc.is_live_pron(999999));
                    }
                }
                $(
                mod when_called_with_a_configured_live_pron_pid {
                    #[test]
                    fn then_true_is_returned() {
                        assert!($live_pron_check);
                    }
                }
                )?
            }
        }
    };
}

process_control_contract!(
    in_memory,
    pron::application::ports::process_control::in_memory::InMemoryProcessControl::with_pid(4242),
    4242u32,
    {
        use pron::application::ports::process_control::ProcessControl;
        let pid = 4242u32;
        let pc = pron::application::ports::process_control::in_memory::InMemoryProcessControl::with_live_pron(pid);
        pc.is_live_pron(pid)
    }
);

process_control_contract!(
    real,
    pron::adapters::process_control::RealProcessControl::new(),
    std::process::id(),
    crate::with_live_pron_daemon(|pid| {
        use pron::application::ports::process_control::ProcessControl;
        pron::adapters::process_control::RealProcessControl::new().is_live_pron(pid)
    })
);

/// Spawns a real, detached `pron -d` daemon, runs `f` while it's confirmed alive
/// (the check must happen inside `f` — the daemon is killed once `f` returns), then kills it.
fn with_live_pron_daemon<T>(f: impl FnOnce(u32) -> T) -> T {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

    let pron = env!("CARGO_BIN_EXE_pron");
    let mut launcher = std::process::Command::new(pron)
        .arg("-d")
        .current_dir(dir.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let status = launcher.wait().unwrap();
    assert!(status.success(), "the daemon should start successfully");

    let pid_str = std::fs::read_to_string(dir.path().join(".pron.pid")).unwrap();
    let daemon_pid: u32 = pid_str.trim().parse().unwrap();

    let result = f(daemon_pid);

    unsafe { libc::kill(daemon_pid as i32, libc::SIGKILL) };
    let _ = std::fs::remove_file(dir.path().join(".pron.pid"));

    result
}

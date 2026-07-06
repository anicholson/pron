use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

mod when_the_pidfile_names_a_stale_pid {
    use super::*;

    #[test]
    fn then_the_pidfile_is_removed_and_pron_stop_exits_zero() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();
        fs::write(dir.path().join(".pron.pid"), "999999\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let output = Command::new(pron)
            .arg("stop")
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "pron stop should exit 0 for a stale pidfile, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !dir.path().join(".pron.pid").exists(),
            ".pron.pid should be removed for a stale pidfile"
        );
    }
}

mod where_proc_is_available {
    use super::*;

    mod when_the_pidfile_names_a_reused_pid_whose_cmdline_is_not_pron {
        use super::*;

        #[test]
        fn then_the_pidfile_is_removed_and_pron_stop_exits_zero_without_signalling() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let mut sleeper = Command::new("sleep")
                .arg("30")
                .spawn()
                .unwrap();
            let reused_pid = sleeper.id();
            fs::write(dir.path().join(".pron.pid"), format!("{reused_pid}\n")).unwrap();

            thread::sleep(Duration::from_millis(100));

            let pron = env!("CARGO_BIN_EXE_pron");
            let output = Command::new(pron)
                .arg("stop")
                .current_dir(dir.path())
                .output()
                .unwrap();

            assert!(
                output.status.success(),
                "pron stop should exit 0 for a reused pid, stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                !dir.path().join(".pron.pid").exists(),
                ".pron.pid should be removed for a reused pid"
            );

            let sleeper_alive = sleeper.try_wait().unwrap().is_none();
            assert!(
                sleeper_alive,
                "the reused pid (sleep) should NOT have been signalled"
            );

            let _ = sleeper.kill();
            let _ = sleeper.wait();
        }
    }
}

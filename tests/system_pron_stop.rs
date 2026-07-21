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

mod when_the_pidfile_names_a_live_pron_daemon {
    use super::*;

    #[test]
    fn then_sigterm_is_delivered_and_the_daemon_removes_the_pidfile_and_exits_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let mut child = Command::new(pron)
            .current_dir(dir.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(500));
        assert!(
            dir.path().join(".pron.pid").exists(),
            "pron should be running with a pidfile"
        );

        let stop_output = Command::new(pron)
            .arg("stop")
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            stop_output.status.success(),
            "pron stop should exit 0, stderr: {}",
            String::from_utf8_lossy(&stop_output.stderr)
        );

        let mut exited = None;
        for _ in 0..100 {
            if let Ok(Some(status)) = child.try_wait() {
                exited = Some(status);
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        let status = exited.expect("pron should exit within 10s of pron stop");
        assert!(
            status.success(),
            "pron should exit cleanly on SIGTERM (exit 0), got {status}"
        );

        assert!(
            !dir.path().join(".pron.pid").exists(),
            ".pron.pid should be removed on clean shutdown"
        );
    }
}

mod if_the_pidfile_does_not_exist {
    use super::*;

    #[test]
    fn then_pron_stop_reports_the_error_and_exits_non_zero() {
        let dir = tempfile::tempdir().unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let output = Command::new(pron)
            .arg("stop")
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "pron stop should exit non-zero when the pidfile does not exist"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(".pron.pid"),
            "stderr should name .pron.pid, got: {stderr}"
        );
    }
}

mod if_the_pidfile_contains_an_invalid_pid {
    use super::*;

    #[test]
    fn then_pron_stop_reports_the_error_and_exits_non_zero() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".pron.pid"), "not-a-pid\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let output = Command::new(pron)
            .arg("stop")
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "pron stop should exit non-zero when the pidfile contains an invalid pid"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("not-a-pid"),
            "stderr should name the invalid pid, got: {stderr}"
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

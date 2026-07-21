use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn wait_for_exit(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Option<std::process::ExitStatus> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Some(status);
        }
        if std::time::Instant::now() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

struct DaemonGuard(std::path::PathBuf);

impl DaemonGuard {
    fn new(dir: &std::path::Path) -> Self {
        Self(dir.to_path_buf())
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let pidfile = self.0.join(".pron.pid");
        if let Ok(pid_str) = fs::read_to_string(&pidfile) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                unsafe { libc::kill(pid, libc::SIGKILL) };
            }
            let _ = fs::remove_file(&pidfile);
        }
    }
}

fn start_daemon(dir: &std::path::Path) -> std::process::Child {
    let pron = env!("CARGO_BIN_EXE_pron");
    Command::new(pron)
        .arg("-d")
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap()
}

mod when_pron_d_is_started_with_a_valid_prontab {
    use super::*;

    #[test]
    fn then_pron_d_exits_zero_once_the_daemon_is_ready() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let _guard = DaemonGuard::new(dir.path());
        let mut child = start_daemon(dir.path());

        let status = wait_for_exit(&mut child, Duration::from_secs(5))
            .expect("pron -d should exit once the daemon is ready");
        assert!(
            status.success(),
            "pron -d should exit 0 once the daemon is ready, got {status}"
        );

        assert!(
            dir.path().join(".pron.pid").exists(),
            ".pron.pid should be written once the daemon is ready"
        );
    }

    #[test]
    fn and_pron_pid_names_the_daemons_pid() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let _guard = DaemonGuard::new(dir.path());
        let mut child = start_daemon(dir.path());
        let launcher_pid = child.id();

        let status = wait_for_exit(&mut child, Duration::from_secs(5))
            .expect("pron -d should exit once the daemon is ready");
        assert!(status.success(), "pron -d should exit 0, got {status}");

        let pid_str = fs::read_to_string(dir.path().join(".pron.pid")).unwrap();
        let daemon_pid: u32 = pid_str.trim().parse().unwrap();

        assert_ne!(
            daemon_pid, launcher_pid,
            ".pron.pid should name the daemon's pid, not the exited launcher's pid {launcher_pid}"
        );
        let alive = unsafe { libc::kill(daemon_pid as i32, 0) == 0 };
        assert!(alive, "the daemon (pid {daemon_pid}) should be running");
    }

    #[test]
    fn and_pron_log_is_appended_to_with_a_start_event() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let _guard = DaemonGuard::new(dir.path());
        let mut child = start_daemon(dir.path());

        let status = wait_for_exit(&mut child, Duration::from_secs(5))
            .expect("pron -d should exit once the daemon is ready");
        assert!(status.success(), "pron -d should exit 0, got {status}");

        let log = fs::read_to_string(dir.path().join(".pron.log")).unwrap();
        assert!(
            log.contains("start"),
            ".pron.log should contain a start event, got: {log}"
        );
    }
}

mod if_the_prontab_has_a_syntax_error {
    use super::*;

    #[test]
    fn then_pron_d_exits_non_zero_and_reports_the_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "not a valid cron line\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let output = Command::new(pron)
            .arg("-d")
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "pron -d should exit non-zero when the crontab has a syntax error"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.to_lowercase().contains("parse") || stderr.to_lowercase().contains("error"),
            "pron -d should report the parse error on stderr, got: {stderr}"
        );
    }

    #[test]
    fn and_no_daemon_is_left_running() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "not a valid cron line\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let output = Command::new(pron)
            .arg("-d")
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "pron -d should exit non-zero when the crontab has a syntax error"
        );
        assert!(
            !dir.path().join(".pron.pid").exists(),
            "no daemon should be left running (no .pron.pid) after a failed start"
        );
    }
}

mod where_proc_is_available {
    use super::*;

    mod when_pron_d_is_started_with_a_valid_prontab {
        use super::*;

        #[test]
        fn then_the_daemon_runs_in_its_own_session_detached_from_the_terminal() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let _guard = DaemonGuard::new(dir.path());
            let mut child = start_daemon(dir.path());

            let status = wait_for_exit(&mut child, Duration::from_secs(5))
                .expect("pron -d should exit once the daemon is ready");
            assert!(status.success(), "pron -d should exit 0, got {status}");

            let pid_str = fs::read_to_string(dir.path().join(".pron.pid")).unwrap();
            let daemon_pid: i32 = pid_str.trim().parse().unwrap();

            let stat = fs::read_to_string(format!("/proc/{daemon_pid}/stat"))
                .expect("/proc/{pid}/stat should be readable for the running daemon");
            let after_comm = stat.rsplit_once(") ").expect("stat should contain comm").1;
            let fields: Vec<&str> = after_comm.split_whitespace().collect();
            let session: i32 = fields[3].parse().expect("session field should parse");

            assert_eq!(
                session, daemon_pid,
                "the daemon should be a session leader in its own session (sid {session} != pid {daemon_pid})"
            );
        }
    }
}

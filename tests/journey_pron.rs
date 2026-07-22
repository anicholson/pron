use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn seconds_until_next_minute() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    60 - (now.as_secs() % 60)
}

fn wait_for_exit(child: &mut std::process::Child, timeout: Duration) -> Option<std::process::ExitStatus> {
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

struct DaemonGuard {
    dir: std::path::PathBuf,
    extra_pids: Vec<i32>,
}

impl DaemonGuard {
    fn new(dir: &std::path::Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
            extra_pids: Vec::new(),
        }
    }

    fn also_kill(&mut self, pid: i32) {
        self.extra_pids.push(pid);
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let pidfile = self.dir.join(".pron.pid");
        if let Ok(pid_str) = fs::read_to_string(&pidfile) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                unsafe { libc::kill(pid, libc::SIGKILL) };
            }
            let _ = fs::remove_file(&pidfile);
        }
        for pid in &self.extra_pids {
            unsafe { libc::kill(*pid, libc::SIGKILL) };
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

mod when_a_prontab_with_a_syntax_error_is_placed_and_pron_d_is_started {
    use super::*;

    #[test]
    fn then_pron_refuses_to_start_and_reports_the_parse_error() {
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
            "pron should refuse to start when the crontab has a syntax error"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.to_lowercase().contains("parse") || stderr.to_lowercase().contains("error"),
            "pron should report the parse error on stderr, got: {stderr}"
        );
    }
}

mod when_the_prontab_is_fixed_with_a_valid_per_minute_job_and_pron_d_is_started {
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
    fn then_pron_pid_is_written_naming_the_daemons_pid() {
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
    fn then_pron_log_is_appended_to_with_a_start_event() {
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

    mod when_a_minute_boundary_is_crossed {
        use super::*;

        #[test]
        fn then_the_jobs_command_runs_in_the_working_directory() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * touch .fired\n").unwrap();

            let _guard = DaemonGuard::new(dir.path());
            let mut child = start_daemon(dir.path());
            let status = wait_for_exit(&mut child, Duration::from_secs(5))
                .expect("pron -d should exit once the daemon is ready");
            assert!(status.success(), "pron -d should exit 0, got {status}");

            let wait = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait));

            let fired = dir.path().join(".fired");
            assert!(
                fired.exists(),
                "job should have run and created .fired in the working directory after {wait}s"
            );
        }

        #[test]
        fn then_the_commands_output_appears_in_pron_log_between_begin_and_end_markers() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let _guard = DaemonGuard::new(dir.path());
            let mut child = start_daemon(dir.path());
            let status = wait_for_exit(&mut child, Duration::from_secs(5))
                .expect("pron -d should exit once the daemon is ready");
            assert!(status.success(), "pron -d should exit 0, got {status}");

            let wait = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait));

            let log = fs::read_to_string(dir.path().join(".pron.log")).unwrap();
            assert!(
                log.contains("--- begin:"),
                ".pron.log should contain a begin marker, got: {log}"
            );
            assert!(
                log.contains("hi"),
                ".pron.log should contain the command output 'hi', got: {log}"
            );
            assert!(
                log.contains("--- end:"),
                ".pron.log should contain an end marker, got: {log}"
            );
        }
    }

    mod when_another_minute_boundary_is_crossed {
        use super::*;

        #[test]
        fn then_the_job_runs_a_second_time() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let _guard = DaemonGuard::new(dir.path());
            let mut child = start_daemon(dir.path());
            let status = wait_for_exit(&mut child, Duration::from_secs(5))
                .expect("pron -d should exit once the daemon is ready");
            assert!(status.success(), "pron -d should exit 0, got {status}");

            let wait1 = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait1));

            thread::sleep(Duration::from_secs(60));

            let log = fs::read_to_string(dir.path().join(".pron.log")).unwrap();
            let begin_count = log.matches("--- begin:").count();
            assert_eq!(
                begin_count, 2,
                "job should have run twice (two begin markers), got {begin_count}: {log}"
            );
        }

        #[test]
        fn and_pron_log_shows_a_second_set_of_begin_and_end_markers() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let _guard = DaemonGuard::new(dir.path());
            let mut child = start_daemon(dir.path());
            let status = wait_for_exit(&mut child, Duration::from_secs(5))
                .expect("pron -d should exit once the daemon is ready");
            assert!(status.success(), "pron -d should exit 0, got {status}");

            let wait1 = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait1));

            thread::sleep(Duration::from_secs(60));

            let log = fs::read_to_string(dir.path().join(".pron.log")).unwrap();
            let begin_count = log.matches("--- begin:").count();
            let end_count = log.matches("--- end:").count();
            assert_eq!(
                (begin_count, end_count),
                (2, 2),
                ".pron.log should show two begin and two end markers, got: {log}"
            );
        }
    }
}

mod when_pron_stop_is_invoked {
    use super::*;

    #[test]
    fn then_the_daemon_receives_sigterm_and_pron_pid_is_removed_and_the_daemon_exits_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let _guard = DaemonGuard::new(dir.path());
        let mut child = start_daemon(dir.path());
        let status = wait_for_exit(&mut child, Duration::from_secs(5))
            .expect("pron -d should exit once the daemon is ready");
        assert!(status.success(), "pron -d should exit 0, got {status}");

        let pid_str = fs::read_to_string(dir.path().join(".pron.pid")).unwrap();
        let daemon_pid: i32 = pid_str.trim().parse().unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let stop_output = Command::new(pron)
            .arg("stop")
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            stop_output.status.success(),
            "pron stop should succeed, stderr: {}",
            String::from_utf8_lossy(&stop_output.stderr)
        );

        assert!(
            !dir.path().join(".pron.pid").exists(),
            ".pron.pid should be removed on clean shutdown"
        );

        let daemon_alive = unsafe { libc::kill(daemon_pid, 0) == 0 };
        assert!(
            !daemon_alive,
            "the daemon (pid {daemon_pid}) should have exited after SIGTERM"
        );
    }
}


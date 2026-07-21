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
}

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

mod when_pron_is_started_while_another_pron_holds_a_live_pidfile {
    use super::*;

    #[test]
    fn then_the_start_is_refused_with_an_error_naming_the_running_pid() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let mut first = Command::new(pron)
            .current_dir(dir.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(500));
        assert!(
            dir.path().join(".pron.pid").exists(),
            "first pron should be running with a pidfile"
        );
        let first_pid = first.id();

        let mut guard = DaemonGuard::new(dir.path());
        guard.also_kill(first_pid as i32);

        let mut second = Command::new(pron)
            .current_dir(dir.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let status = wait_for_exit(&mut second, Duration::from_secs(5));
        if status.is_none() {
            let _ = second.kill();
            let _ = second.wait();
        }
        let status = status.expect("the second pron should exit promptly when refused");
        assert!(
            !status.success(),
            "the second pron should be refused while the first is running, got {status}"
        );

        let mut stderr = String::new();
        if let Some(mut err) = second.stderr.take() {
            use std::io::Read;
            let _ = err.read_to_string(&mut stderr);
        }
        assert!(
            stderr.contains(&first_pid.to_string()),
            "the refusal should name the running pid {first_pid}, got: {stderr}"
        );
    }

    #[test]
    fn and_the_pidfile_is_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let mut first = Command::new(pron)
            .current_dir(dir.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(500));
        let first_pid = first.id();

        let mut guard = DaemonGuard::new(dir.path());
        guard.also_kill(first_pid as i32);

        let second = Command::new(pron)
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            !second.status.success(),
            "the second pron should be refused while the first is running"
        );

        let pidfile = fs::read_to_string(dir.path().join(".pron.pid")).unwrap();
        assert_eq!(
            pidfile.trim().parse::<u32>().unwrap(),
            first_pid,
            "the pidfile should still name the first pron's pid {first_pid}, got: {pidfile}"
        );
    }

    #[test]
    fn and_the_first_pron_keeps_running() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let mut first = Command::new(pron)
            .current_dir(dir.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(500));
        let first_pid = first.id();

        let mut guard = DaemonGuard::new(dir.path());
        guard.also_kill(first_pid as i32);

        let second = Command::new(pron)
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            !second.status.success(),
            "the second pron should be refused while the first is running"
        );

        assert!(
            first.try_wait().unwrap().is_none(),
            "the first pron should still be running after the refused start"
        );
    }
}

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

mod when_pron_is_started_without_d {
    use super::*;

    #[test]
    fn then_pron_pid_is_written() {
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
            ".pron.pid should be written in foreground mode"
        );

        child.kill().unwrap();
        let _ = child.wait();
    }

    #[test]
    fn then_the_start_event_is_printed_to_stdout() {
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

        child.kill().unwrap();
        let _ = child.wait();

        let mut stdout = String::new();
        if let Some(mut out) = child.stdout.take() {
            use std::io::Read;
            let _ = out.read_to_string(&mut stdout);
        }

        assert!(
            stdout.contains("start"),
            "stdout should contain the start event, got: {stdout}"
        );
    }

    #[test]
    fn then_no_pron_log_is_created() {
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
            !dir.path().join(".pron.log").exists(),
            ".pron.log should NOT be created in foreground mode"
        );

        child.kill().unwrap();
        let _ = child.wait();
    }

    mod when_a_minute_boundary_is_crossed {
        use super::*;

        #[test]
        fn then_the_jobs_command_runs_in_the_working_directory() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * touch .fired\n").unwrap();

            let pron = env!("CARGO_BIN_EXE_pron");
            let mut child = Command::new(pron)
                .current_dir(dir.path())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();

            let wait = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait));

            assert!(
                dir.path().join(".fired").exists(),
                "job should have run and created .fired after {wait}s"
            );

            child.kill().unwrap();
            let _ = child.wait();
        }

        #[test]
        fn then_the_commands_output_flows_to_stdout_with_no_markers() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let pron = env!("CARGO_BIN_EXE_pron");
            let mut child = Command::new(pron)
                .current_dir(dir.path())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();

            let wait = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait));

            let mut stdout = String::new();
            if let Some(mut out) = child.stdout.take() {
                use std::io::Read;
                let _ = out.read_to_string(&mut stdout);
            }

            assert!(
                stdout.contains("hi"),
                "stdout should contain the command output 'hi', got: {stdout}"
            );
            assert!(
                !stdout.contains("--- begin:"),
                "stdout should NOT contain begin markers in foreground mode, got: {stdout}"
            );
            assert!(
                !stdout.contains("--- end:"),
                "stdout should NOT contain end markers in foreground mode, got: {stdout}"
            );

            child.kill().unwrap();
            let _ = child.wait();
        }
    }
}

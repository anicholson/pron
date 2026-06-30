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
    fn then_the_daemon_starts() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let mut child = Command::new(pron)
            .arg("-d")
            .current_dir(dir.path())
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(500));

        let pidfile = dir.path().join(".pron.pid");
        assert!(
            pidfile.exists(),
            ".pron.pid should be written when the daemon starts"
        );

        let log = dir.path().join(".pron.log");
        assert!(
            log.exists(),
            ".pron.log should be created when the daemon starts"
        );
        let log_content = fs::read_to_string(&log).unwrap();
        assert!(
            log_content.contains("start"),
            ".pron.log should contain a start event, got: {log_content}"
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
                .arg("-d")
                .current_dir(dir.path())
                .spawn()
                .unwrap();

            let wait = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait));

            let fired = dir.path().join(".fired");
            assert!(
                fired.exists(),
                "job should have run and created .fired in the working directory after {wait}s"
            );

            child.kill().unwrap();
            let _ = child.wait();
        }

        #[test]
        fn then_the_commands_output_appears_in_pron_log_between_begin_and_end_markers() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let pron = env!("CARGO_BIN_EXE_pron");
            let mut child = Command::new(pron)
                .arg("-d")
                .current_dir(dir.path())
                .spawn()
                .unwrap();

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

            child.kill().unwrap();
            let _ = child.wait();
        }
    }

    mod when_another_minute_boundary_is_crossed {
        use super::*;

        #[test]
        fn then_the_job_runs_a_second_time() {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

            let pron = env!("CARGO_BIN_EXE_pron");
            let mut child = Command::new(pron)
                .arg("-d")
                .current_dir(dir.path())
                .spawn()
                .unwrap();

            let wait1 = super::super::seconds_until_next_minute() + 3;
            thread::sleep(Duration::from_secs(wait1));

            thread::sleep(Duration::from_secs(60));

            let log = fs::read_to_string(dir.path().join(".pron.log")).unwrap();
            let begin_count = log.matches("--- begin:").count();
            assert_eq!(
                begin_count, 2,
                "job should have run twice (two begin markers), got {begin_count}: {log}"
            );

            child.kill().unwrap();
            let _ = child.wait();
        }
    }
}

mod when_pron_stop_is_invoked {
    use super::*;

    #[test]
    fn then_the_daemon_receives_sigterm_and_pron_pid_is_removed_and_the_daemon_exits_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".prontab"), "* * * * * echo hi\n").unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let mut child = Command::new(pron)
            .arg("-d")
            .current_dir(dir.path())
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(500));
        assert!(
            dir.path().join(".pron.pid").exists(),
            "daemon should be running with pidfile"
        );

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

        let status = child.wait().unwrap();
        assert!(
            status.success(),
            "daemon should exit cleanly (exit 0), got {status}"
        );

        assert!(
            !dir.path().join(".pron.pid").exists(),
            ".pron.pid should be removed on clean shutdown"
        );
    }
}


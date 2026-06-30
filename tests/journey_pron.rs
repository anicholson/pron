use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

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
}


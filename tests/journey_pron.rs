use std::fs;
use std::process::Command;
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

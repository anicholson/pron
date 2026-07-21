use std::process::Command;

mod when_pron_is_started_without_a_readable_prontab {
    use super::*;

    #[test]
    fn then_pron_reports_the_read_error_and_exits_non_zero() {
        let dir = tempfile::tempdir().unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let output = Command::new(pron)
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "pron should exit non-zero without a readable .prontab"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(".prontab"),
            "stderr should name .prontab, got: {stderr}"
        );
    }

    #[test]
    fn and_no_pron_pid_is_written() {
        let dir = tempfile::tempdir().unwrap();

        let pron = env!("CARGO_BIN_EXE_pron");
        let _ = Command::new(pron)
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            !dir.path().join(".pron.pid").exists(),
            ".pron.pid should not be written when the .prontab cannot be read"
        );
    }
}

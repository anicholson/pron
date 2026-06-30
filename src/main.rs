use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pron::adapters::clock::SystemClock;
use pron::adapters::fs::RealFilesystem;
use pron::adapters::logger::RealLogger;
use pron::adapters::process_control::RealProcessControl;
use pron::adapters::process_runner::ShProcessRunner;
use pron::application::scheduler::Scheduler;
use pron::application::start::Start;

fn main() {
    let cwd: PathBuf = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("error: cannot determine current directory: {e}");
        std::process::exit(1);
    });

    let content = match std::fs::read_to_string(cwd.join(".prontab")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read .prontab: {e}");
            std::process::exit(1);
        }
    };

    let args: Vec<String> = std::env::args().collect();
    let daemon = args.iter().any(|a| a == "-d" || a == "--daemon");
    let mode = if daemon { "daemon" } else { "foreground" };

    let fs = RealFilesystem::new(&cwd);
    let logger = RealLogger::new(&cwd);
    let proc = RealProcessControl::new();
    let start = Start::new(fs, logger, proc);

    let entries = match start.execute(&content, mode) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let clock = SystemClock::new();
    let runner = ShProcessRunner::new();
    let scheduler = Scheduler::new(clock, runner, entries);

    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let secs_remaining = 60 - (now % 60);
        std::thread::sleep(Duration::from_secs(secs_remaining));
        scheduler.tick();
    }
}
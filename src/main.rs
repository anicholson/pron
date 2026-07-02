use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pron::adapters::clock::SystemClock;
use pron::adapters::fs::RealFilesystem;
use pron::adapters::logger::RealLogger;
use pron::adapters::process_control::RealProcessControl;
use pron::adapters::process_runner::ShProcessRunner;
use pron::application::ports::filesystem::Filesystem;
use pron::application::scheduler::Scheduler;
use pron::application::start::Start;

fn main() {
    let cwd: PathBuf = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("error: cannot determine current directory: {e}");
        std::process::exit(1);
    });

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "stop") {
        do_stop(&cwd);
        return;
    }

    let content = match std::fs::read_to_string(cwd.join(".prontab")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read .prontab: {e}");
            std::process::exit(1);
        }
    };

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

    let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, shutdown.clone()).unwrap();
    signal_hook::flag::register(signal_hook::consts::SIGINT, shutdown.clone()).unwrap();

    let clock = SystemClock::new();
    let runner = ShProcessRunner::new();
    let log_for_scheduler = RealLogger::new(&cwd);
    let scheduler = Scheduler::new(clock, runner, log_for_scheduler, entries);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            let fs = RealFilesystem::new(&cwd);
            let _ = fs.remove_pidfile();
            std::process::exit(0);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let secs_remaining = (60 - (now % 60)) % 60;
        let sleep_until = SystemTime::now() + Duration::from_secs(secs_remaining);
        while SystemTime::now() < sleep_until {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }

        if !shutdown.load(Ordering::Relaxed) {
            scheduler.tick();
        }
    }
}

fn do_stop(cwd: &Path) {
    let pid_str = match std::fs::read_to_string(cwd.join(".pron.pid")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read .pron.pid: {e}");
            std::process::exit(1);
        }
    };

    let pid: i32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("error: .pron.pid contains invalid pid: {pid_str}");
            std::process::exit(1);
        }
    };

    unsafe {
        if libc::kill(pid, libc::SIGTERM) != 0 {
            eprintln!("warning: stale pidfile (pid {pid} not alive), removing");
            let _ = std::fs::remove_file(cwd.join(".pron.pid"));
            std::process::exit(0);
        }
    }

    for _ in 0..50 {
        unsafe {
            if libc::kill(pid, 0) != 0 {
                std::process::exit(0);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    eprintln!("warning: pid {pid} still alive after 5s");
    std::process::exit(0);
}
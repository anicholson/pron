use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pron::adapters::clock::SystemClock;
use pron::adapters::fs::RealFilesystem;
use pron::adapters::logger::RealLogger;
use pron::adapters::process_control::RealProcessControl;
use pron::adapters::process_runner::ShProcessRunner;
use pron::adapters::stdout_logger::StdoutLogger;
use pron::application::ports::filesystem::Filesystem;
use pron::application::ports::logger::Logger;
use pron::application::scheduler::Scheduler;
use pron::application::start::Start;
use pron::domain::crontab;

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

    if args.iter().any(|a| a == "-d" || a == "--daemon") {
        run_daemon(&cwd, &content);
    }

    let fs = RealFilesystem::new(&cwd);
    let proc = RealProcessControl::new();
    let logger = StdoutLogger::new();
    let entries = match Start::new(fs, logger, proc).execute(&content, "foreground") {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    run_scheduler(&cwd, entries, Box::new(StdoutLogger::new()));
}

fn run_daemon(cwd: &Path, content: &str) -> ! {
    if let Err(e) = crontab::parse(content) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }

    let mut fds = [0i32; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        eprintln!(
            "error: cannot create pipe: {}",
            std::io::Error::last_os_error()
        );
        std::process::exit(1);
    }

    match unsafe { libc::fork() } {
        -1 => {
            eprintln!("error: cannot fork: {}", std::io::Error::last_os_error());
            std::process::exit(1);
        }
        0 => {
            unsafe { libc::close(fds[0]) };
            daemon_child(cwd, content, fds[1]);
        }
        _ => {
            unsafe { libc::close(fds[1]) };
            let mut msg = Vec::new();
            loop {
                let mut buf = [0u8; 1024];
                let n = unsafe {
                    libc::read(fds[0], buf.as_mut_ptr() as *mut libc::c_void, buf.len())
                };
                if n <= 0 {
                    break;
                }
                msg.extend_from_slice(&buf[..n as usize]);
            }
            unsafe { libc::close(fds[0]) };

            let msg = String::from_utf8_lossy(&msg);
            if msg.starts_with("ok") {
                std::process::exit(0);
            }
            let detail = msg.strip_prefix("err: ").unwrap_or(&msg);
            eprintln!("error: {}", detail.trim_end());
            std::process::exit(1);
        }
    }
}

fn daemon_child(cwd: &Path, content: &str, ready_fd: i32) -> ! {
    unsafe { libc::setsid() };

    let devnull = unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_RDWR) };
    if devnull >= 0 {
        unsafe {
            libc::dup2(devnull, 0);
            libc::dup2(devnull, 1);
            libc::dup2(devnull, 2);
            if devnull > 2 {
                libc::close(devnull);
            }
        }
    }

    let fs = RealFilesystem::new(cwd);
    let logger = RealLogger::new(cwd);
    let proc = RealProcessControl::new();
    match Start::new(fs, logger, proc).execute(content, "daemon") {
        Ok(entries) => {
            write_ready(ready_fd, "ok");
            unsafe { libc::close(ready_fd) };
            run_scheduler(cwd, entries, Box::new(RealLogger::new(cwd)));
        }
        Err(e) => {
            write_ready(ready_fd, &format!("err: {e}"));
            unsafe { libc::close(ready_fd) };
            std::process::exit(1);
        }
    }
}

fn write_ready(fd: i32, msg: &str) {
    let bytes = msg.as_bytes();
    unsafe {
        libc::write(fd, bytes.as_ptr() as *const libc::c_void, bytes.len());
    }
}

fn run_scheduler(cwd: &Path, entries: Vec<crontab::Entry>, logger: Box<dyn Logger>) -> ! {
    let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, shutdown.clone()).unwrap();
    signal_hook::flag::register(signal_hook::consts::SIGINT, shutdown.clone()).unwrap();

    let clock = SystemClock::new();
    let runner = ShProcessRunner::new();
    let scheduler = Scheduler::new(clock, runner, logger, entries);

    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let secs_remaining = 60 - (now % 60);
        let sleep_until = SystemTime::now() + Duration::from_secs(secs_remaining);
        while SystemTime::now() < sleep_until {
            if shutdown.load(Ordering::Relaxed) {
                let fs = RealFilesystem::new(cwd);
                let _ = fs.remove_pidfile();
                std::process::exit(0);
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

    if !RealProcessControl::process_is_alive(pid) {
        eprintln!("warning: stale pidfile (pid {pid} not alive), removing");
        let _ = std::fs::remove_file(cwd.join(".pron.pid"));
        std::process::exit(0);
    }

    if !RealProcessControl::looks_like_pron(pid) {
        eprintln!("warning: stale pidfile (pid {pid} is not pron), removing");
        let _ = std::fs::remove_file(cwd.join(".pron.pid"));
        std::process::exit(0);
    }

    unsafe {
        if libc::kill(pid, libc::SIGTERM) != 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::ESRCH) {
                eprintln!("warning: stale pidfile (pid {pid} not alive), removing");
                let _ = std::fs::remove_file(cwd.join(".pron.pid"));
                std::process::exit(0);
            }
            eprintln!("error: cannot signal pid {pid}: {err}");
            std::process::exit(1);
        }
    }

    for _ in 0..50 {
        if !RealProcessControl::process_is_alive(pid) {
            std::process::exit(0);
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    eprintln!("warning: pid {pid} still alive after 5s");
    std::process::exit(0);
}

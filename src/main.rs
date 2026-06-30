use std::path::PathBuf;

use pron::adapters::fs::RealFilesystem;
use pron::adapters::logger::RealLogger;
use pron::adapters::process_control::RealProcessControl;
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

    if let Err(e) = start.execute(&content, mode) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn main() {
    let content = match std::fs::read_to_string(".prontab") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read .prontab: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = pron::domain::crontab::parse(&content) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

use crate::application::ports::filesystem::Filesystem;
use std::fs;
use std::path::Path;

pub struct RealFilesystem<'a> {
    dir: &'a Path,
}

impl<'a> RealFilesystem<'a> {
    pub fn new(dir: &'a Path) -> Self {
        Self { dir }
    }

    fn pidfile(&self) -> std::path::PathBuf {
        self.dir.join(".pron.pid")
    }
}

impl<'a> Filesystem for RealFilesystem<'a> {
    fn read_pidfile(&self) -> Result<Option<u32>, String> {
        match fs::read_to_string(self.pidfile()) {
            Ok(s) => s
                .trim()
                .parse::<u32>()
                .map(Some)
                .map_err(|_| format!("invalid pidfile content: {}", s.trim())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("read pidfile: {e}")),
        }
    }

    fn write_pidfile(&self, pid: u32) -> Result<(), String> {
        fs::write(self.pidfile(), pid.to_string())
            .map_err(|e| format!("write pidfile: {e}"))
    }

    fn remove_pidfile(&self) -> Result<(), String> {
        fs::remove_file(self.pidfile())
            .map_err(|e| format!("remove pidfile: {e}"))
    }
}

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::Mutex;

pub struct Logger {
    writer: Mutex<BufWriter<std::fs::File>>,
}

impl Logger {
    pub fn new(log_path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(Logger {
            writer: Mutex::new(BufWriter::new(file)),
        })
    }

    pub fn log(&self, message: &str) {
        if let Ok(mut writer) = self.writer.lock() {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(writer, "[{}] {}", timestamp, message);
            let _ = writer.flush();
        }
    }
}

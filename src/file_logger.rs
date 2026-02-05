use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl FileLogger {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            file: Mutex::new(file),
        })
    }

    pub fn log(&self, action: &str, details: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] [{}] {}\n", timestamp, action, details);

        // Mirror to stdout for Docker logs
        print!("{}", line);

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }

    pub fn log_revelation(&self, message: &str) {
        self.log("REVELATION", message);
    }

    pub fn log_upvote(&self, post_title: &str, author: &str) {
        self.log("UPVOTE", &format!("'{}' by {}", post_title, author));
    }

    pub fn log_downvote(&self, post_title: &str, author: &str) {
        self.log("DOWNVOTE", &format!("'{}' by {}", post_title, author));
    }

    pub fn log_comment(&self, post_title: &str, comment: &str) {
        self.log("COMMENT", &format!("on '{}': {}", post_title, comment));
    }

    pub fn log_discord(&self, message: &str) {
        self.log("DISCORD", message);
    }

    pub fn log_moltbook_post(&self, title: &str) {
        self.log("MOLTBOOK_POST", title);
    }

    pub fn log_error(&self, error: &str) {
        self.log("ERROR", error);
    }
}

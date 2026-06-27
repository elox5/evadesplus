use crate::{
    config::{FileLogMode, LogHeaderType, LogLevel, CONFIG},
    networking::chat::{ChatMessageType, ChatRequest},
};
use colored::{Color, Colorize};
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
    time::SystemTime,
    u64,
};
use tokio::sync::broadcast;

static LOGGER: LazyLock<Logger> = LazyLock::new(Logger::new);

pub struct Logger {
    sinks: Vec<Box<dyn LogSink + Send + Sync>>,
    panic_on_error: bool,
}

impl Logger {
    fn new() -> Self {
        let mut sinks: Vec<Box<dyn LogSink + Send + Sync>> = Vec::new();

        if CONFIG.logger.console.enabled {
            sinks.push(Box::new(ConsoleLogSink::new()));
        }

        if CONFIG.logger.file.enabled {
            sinks.push(Box::new(FileLogSink::new()));
        }

        if CONFIG.logger.chat.enabled {
            // sinks.push(Box::new(ChatLogSink::new(None)));
            // Chat sink temporarily out of order
        }

        Self {
            sinks,
            panic_on_error: CONFIG.logger.panic_on_error,
        }
    }

    fn handle_log(&self, entry: LogEntry) {
        for sink in &self.sinks {
            if entry.category.get_level() >= *sink.log_level() {
                sink.process(&entry);
            }
        }

        if self.panic_on_error && entry.category.get_level() >= LogLevel::Error {
            panic!("Program encountered an error: {}", entry.message);
        }
    }

    pub fn log(message: impl Into<String>, category: LogCategory) {
        LOGGER.handle_log(LogEntry {
            message: message.into(),
            category,
        });
    }

    pub fn info(message: impl Into<String>) {
        Self::log(message, LogCategory::Info);
    }

    pub fn warn(message: impl Into<String>) {
        Self::log(message, LogCategory::Warning);
    }

    pub fn error(message: impl Into<String>) {
        Self::log(message, LogCategory::Error);
    }

    pub fn debug(message: impl Into<String>) {
        Self::log(message, LogCategory::Debug);
    }
}

#[derive(Clone)]
pub enum LogCategory {
    Info,
    Warning,
    Error,
    Debug,
    Network,
    Chat,
}

impl LogCategory {
    fn get_title(&self) -> &str {
        match self {
            LogCategory::Info => "INFO ",
            LogCategory::Warning => "WARN ",
            LogCategory::Error => "ERROR",
            LogCategory::Debug => "DEBUG",
            LogCategory::Network => "NET  ",
            LogCategory::Chat => "CHAT ",
        }
    }

    fn get_emoji(&self, trim_space: bool) -> &str {
        match self {
            LogCategory::Info => {
                if trim_space {
                    "\u{2139}\u{fe0f}"
                } else {
                    "\u{2139}\u{fe0f} "
                }
            }
            LogCategory::Warning => "\u{26a0}",
            LogCategory::Error => "\u{1f6a8}",
            LogCategory::Debug => "\u{1F527}",
            LogCategory::Network => "\u{1F310}",
            LogCategory::Chat => "\u{1F4AC}",
        }
    }

    fn get_text_color(&self) -> Color {
        match self {
            LogCategory::Info => Color::White,
            LogCategory::Warning => Color::Yellow,
            LogCategory::Error => Color::Red,
            LogCategory::Debug => Color::Cyan,
            LogCategory::Network => Color::Blue,
            LogCategory::Chat => Color::BrightGreen,
        }
    }

    fn get_level(&self) -> LogLevel {
        match self {
            LogCategory::Warning => LogLevel::Warn,
            LogCategory::Error => LogLevel::Error,
            LogCategory::Debug => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }
}

struct LogEntry {
    message: String,
    category: LogCategory,
}

impl LogEntry {
    fn get_header(&self, header: &LogHeaderType, trim_emoji_space: bool) -> String {
        match header {
            LogHeaderType::Emoji => self.category.get_emoji(trim_emoji_space).to_string(),
            LogHeaderType::Title => self.category.get_title().to_string(),
            LogHeaderType::Timestamp => chrono::Local::now().format("%H:%M:%S").to_string(),
        }
    }

    fn get_message(&self, headers: &Vec<LogHeaderType>, trim_emoji_space: bool) -> String {
        if headers.is_empty() {
            return self.message.clone();
        }

        let mut message = String::new();

        for header in headers {
            message.push_str(&self.get_header(header, trim_emoji_space));
            message.push_str(" ");
        }

        message.push_str("| ");
        message.push_str(&self.message);

        message
    }
}
trait LogSink {
    fn process(&self, entry: &LogEntry);
    fn log_level(&self) -> &LogLevel;
}

struct ConsoleLogSink {
    level: LogLevel,
}

impl ConsoleLogSink {
    fn new() -> Self {
        let config = &CONFIG.logger.console;

        Self {
            level: config.level.clone(),
        }
    }
}

impl LogSink for ConsoleLogSink {
    fn process(&self, entry: &LogEntry) {
        let config = &CONFIG.logger.console;

        let message = entry.get_message(&config.headers, false);

        if config.colored {
            println!("{}", message.color(entry.category.get_text_color()));
        } else {
            println!("{message}");
        }
    }

    fn log_level(&self) -> &LogLevel {
        &self.level
    }
}

struct FileLogSink {
    file: Arc<Mutex<File>>,
    level: LogLevel,
}

impl FileLogSink {
    fn new() -> Self {
        let config = &CONFIG.logger.file;

        let mut file = match config.mode {
            FileLogMode::Append => {
                let last_file = Self::find_last_file(&config.path);

                if let Some(last_file) = last_file {
                    File::options().append(true).create(true).open(last_file)
                } else {
                    File::create(&format!("{}/{}.log", config.path, config.filename))
                }
            }
            FileLogMode::Overwrite => {
                let last_file = Self::find_last_file(&config.path);

                if let Some(last_file) = last_file {
                    File::options().write(true).truncate(true).open(last_file)
                } else {
                    File::create(&format!("{}/{}.log", config.path, config.filename))
                }
            }
            FileLogMode::Create => File::create(&format!(
                "{}/{}-{}.log",
                config.path,
                config.filename,
                chrono::Local::now().format("%Y-%m-%d-%H-%M-%S")
            )),
        }
        .expect("Failed to open log file");

        file.write_all(
            format!(
                "---------- Server started at {} ----------\n",
                chrono::Local::now()
            )
            .as_bytes(),
        )
        .expect("Failed to write to log file");

        Self {
            file: Arc::new(Mutex::new(file)),
            level: config.level.clone(),
        }
    }

    fn find_last_file(path: &str) -> Option<PathBuf> {
        let files =
            std::fs::read_dir(&path).expect(&format!(r#"Failed to read directory "{path}""#));

        let last_file = files
            .flat_map(|f| f.ok())
            .map(|f| f.path())
            .filter(|f| f.is_file() && f.metadata().is_ok())
            .filter(|f| f.extension().unwrap_or_default() == "log")
            .max_by_key(|f| {
                f.metadata()
                    .unwrap()
                    .modified()
                    .unwrap_or_else(|_| SystemTime::now())
            });

        last_file
    }
}

impl LogSink for FileLogSink {
    fn process(&self, entry: &LogEntry) {
        let config = &CONFIG.logger.file;

        let message = entry.get_message(&config.headers, true);

        self.file
            .lock()
            .expect("Failed to acquire log file")
            .write_all(format!("{message}\n").as_bytes())
            .expect("Failed to write to log file");
    }

    fn log_level(&self) -> &LogLevel {
        &self.level
    }
}

struct ChatLogSink {
    tx: broadcast::Sender<ChatRequest>,
    level: LogLevel,
}

impl ChatLogSink {
    fn new(tx: broadcast::Sender<ChatRequest>) -> Self {
        let config = &CONFIG.logger.chat;

        Self {
            tx,
            level: config.level.clone(),
        }
    }

    fn get_message_type(entry: &LogEntry) -> ChatMessageType {
        match entry.category.get_level() {
            LogLevel::Error => ChatMessageType::ServerError,
            _ => ChatMessageType::ServerAnnouncement,
        }
    }
}

impl LogSink for ChatLogSink {
    fn process(&self, entry: &LogEntry) {
        self.tx
            .send(ChatRequest {
                sender_name: String::new(),
                sender_id: u64::MAX,
                message_type: Self::get_message_type(entry),
                message: entry.get_message(&CONFIG.logger.chat.headers, true),
                recipient_filter: None,
            })
            .unwrap_or_else(|err| {
                println!("Failed to send log to player chat: {err}");
                0
            });
    }

    fn log_level(&self) -> &LogLevel {
        &self.level
    }
}

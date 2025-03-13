use crate::config::{FileLogMode, LogHeaderType, LogLevel, CONFIG};
use colored::{Color, Colorize};
use std::{
    fs::File,
    io::Write,
    sync::{Arc, LazyLock, Mutex},
};

static LOGGER: LazyLock<Logger> = LazyLock::new(Logger::new);

pub struct Logger {
    handlers: Vec<Box<dyn Handler + Send + Sync>>,
}

impl Logger {
    fn new() -> Self {
        let mut handlers: Vec<Box<dyn Handler + Send + Sync>> = Vec::new();

        if CONFIG.logger.console.enabled {
            handlers.push(Box::new(ConsoleHandler));
        }

        if CONFIG.logger.file.enabled {
            handlers.push(Box::new(FileHandler::new()));
        }

        Self { handlers }
    }

    fn handle_log(&self, entry: LogEntry) {
        for handler in &self.handlers {
            handler.handle(&entry);
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

pub enum LogCategory {
    Info,
    Warning,
    Error,
    Debug,
    Network,
    Chat,
}

impl LogCategory {
    fn get_header(&self) -> &str {
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
            LogHeaderType::Text => self.category.get_header().to_string(),
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
trait Handler {
    fn handle(&self, entry: &LogEntry);
}

struct ConsoleHandler;

impl Handler for ConsoleHandler {
    fn handle(&self, entry: &LogEntry) {
        let config = &CONFIG.logger.console;

        if entry.category.get_level() < config.level {
            return;
        }

        let message = entry.get_message(&config.headers, false);

        if config.colored {
            println!("{}", message.color(entry.category.get_text_color()));
        } else {
            println!("{message}");
        }
    }
}

struct FileHandler {
    file: Arc<Mutex<File>>,
}

impl FileHandler {
    fn new() -> Self {
        let config = &CONFIG.logger.file;

        let mut file = match config.mode {
            FileLogMode::Append => File::options().append(true).create(true).open(&config.path),
            FileLogMode::Overwrite => File::create(&config.path),
        }
        .expect("Failed to open log file");

        file.write_all(
            format!(
                "---------- Server starting at {} ----------\n",
                chrono::Local::now()
            )
            .as_bytes(),
        )
        .expect("Failed to write to log file");

        Self {
            file: Arc::new(Mutex::new(file)),
        }
    }
}

impl Handler for FileHandler {
    fn handle(&self, entry: &LogEntry) {
        let config = &CONFIG.logger.file;

        if entry.category.get_level() < config.level {
            return;
        }

        let message = entry.get_message(&config.headers, true);

        self.file
            .lock()
            .expect("Failed to acquire log file")
            .write_all(format!("{message}\n").as_bytes())
            .expect("Failed to write to log file");
    }
}

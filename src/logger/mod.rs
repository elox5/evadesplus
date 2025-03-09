use std::sync::LazyLock;

use colored::Colorize;

static LOGGER: LazyLock<Logger> = LazyLock::new(Logger::new);

pub struct Logger {
    handlers: Vec<Box<dyn Handler + Send + Sync>>,
}

impl Logger {
    fn new() -> Self {
        let console_handler = ConsoleHandler { colored: true };

        Self {
            handlers: vec![Box::new(console_handler)],
        }
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
}

impl LogCategory {
    fn get_header(&self) -> &str {
        match self {
            LogCategory::Info => "INFO ",
            LogCategory::Warning => "WARN ",
            LogCategory::Error => "ERROR",
            LogCategory::Debug => "DEBUG",
        }
    }

    fn get_emoji(&self) -> &str {
        match self {
            LogCategory::Info => "\u{2139}\u{fe0f} ",
            LogCategory::Warning => "\u{26a0}",
            LogCategory::Error => "\u{1f6a8}",
            LogCategory::Debug => "\u{1F527}",
        }
    }

    fn color_message(&self, message: &str) -> String {
        match self {
            LogCategory::Info => message.to_owned(),
            LogCategory::Warning => message.yellow().to_string(),
            LogCategory::Error => message.red().to_string(),
            LogCategory::Debug => message.cyan().to_string(),
        }
    }
}

struct LogEntry {
    message: String,
    category: LogCategory,
}

trait Handler {
    fn handle(&self, entry: &LogEntry);
}

struct ConsoleHandler {
    colored: bool,
}

impl Handler for ConsoleHandler {
    fn handle(&self, entry: &LogEntry) {
        let message = format!(
            "{} {} | {}",
            entry.category.get_emoji(),
            entry.category.get_header(),
            entry.message
        );

        if self.colored {
            println!("{}", entry.category.color_message(&message));
        } else {
            println!("{message}");
        }
    }
}

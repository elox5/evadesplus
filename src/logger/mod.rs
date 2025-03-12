use std::sync::LazyLock;

use colored::{Color, Colorize};

use crate::config::{LogHeaderType, CONFIG};

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

    fn get_emoji(&self) -> &str {
        match self {
            LogCategory::Info => "\u{2139}\u{fe0f} ",
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
}

struct LogEntry {
    message: String,
    category: LogCategory,
}

trait Handler {
    fn handle(&self, entry: &LogEntry);
}

struct ConsoleHandler;

impl Handler for ConsoleHandler {
    fn handle(&self, entry: &LogEntry) {
        let config = &CONFIG.logger.console;

        let header = match config.header_type {
            LogHeaderType::None => String::new(),
            LogHeaderType::Emoji => format!("{} |", entry.category.get_emoji()),
            LogHeaderType::Text => format!("{} |", entry.category.get_header()),
            LogHeaderType::Full => format!(
                "{} {} |",
                entry.category.get_emoji(),
                entry.category.get_header()
            ),
        };

        let message = format!("{} {}", header, entry.message);

        if config.colored {
            println!("{}", message.color(entry.category.get_text_color()));
        } else {
            println!("{message}");
        }
    }
}

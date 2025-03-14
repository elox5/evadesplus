use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, sync::LazyLock};

pub const CONFIG: LazyLock<Config> = LazyLock::new(init_config);

fn init_config() -> Config {
    Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config.toml"))
        .extract()
        .unwrap()
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub maps: MapConfig,
    #[serde(default)]
    pub game: GameConfig,
    #[serde(default)]
    pub logger: LoggerConfig,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkConfig {
    pub ip: Ipv4Addr,
    pub client_port_https: u16,
    pub client_port_http: u16,
    pub webtransport_port: u16,
    pub client_path: String,
    pub ssl_cert_path: String,
    pub ssl_key_path: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            client_port_https: 3333,
            client_port_http: 3000,
            webtransport_port: 3334,
            client_path: String::from("client/dist"),
            ssl_cert_path: String::from("ssl/cert.pem"),
            ssl_key_path: String::from("ssl/key.pem"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MapConfig {
    pub path: String,
    pub maps: Option<Vec<String>>,
}

impl Default for MapConfig {
    fn default() -> Self {
        MapConfig {
            path: String::from("maps"),
            maps: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameConfig {
    pub simulation_framerate: f32,
    pub spawn_map: Option<String>,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            simulation_framerate: 60.0,
            spawn_map: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum LogHeaderType {
    Emoji,
    Title,
    Timestamp,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

#[derive(Serialize, Deserialize)]
pub enum FileLogMode {
    Append,
    Overwrite,
    Create,
}

#[derive(Serialize, Deserialize, Default)]
pub struct LoggerConfig {
    pub console: LoggerConsoleConfig,
    pub file: LoggerFileConfig,
    pub chat: LoggerChatConfig,
}

#[derive(Serialize, Deserialize)]
pub struct LoggerConsoleConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub headers: Vec<LogHeaderType>,
    pub colored: bool,
}

impl Default for LoggerConsoleConfig {
    fn default() -> Self {
        LoggerConsoleConfig {
            enabled: true,
            level: LogLevel::Info,
            headers: vec![
                LogHeaderType::Timestamp,
                LogHeaderType::Emoji,
                LogHeaderType::Title,
            ],
            colored: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoggerFileConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub headers: Vec<LogHeaderType>,
    pub path: String,
    pub filename: String,
    pub mode: FileLogMode,
}

impl Default for LoggerFileConfig {
    fn default() -> Self {
        LoggerFileConfig {
            enabled: true,
            level: LogLevel::Info,
            headers: vec![
                LogHeaderType::Timestamp,
                LogHeaderType::Emoji,
                LogHeaderType::Title,
            ],
            path: String::from("logs/"),
            filename: String::from("server"),
            mode: FileLogMode::Append,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoggerChatConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub headers: Vec<LogHeaderType>,
}

impl Default for LoggerChatConfig {
    fn default() -> Self {
        LoggerChatConfig {
            enabled: true,
            level: LogLevel::Warn,
            headers: vec![LogHeaderType::Title],
        }
    }
}

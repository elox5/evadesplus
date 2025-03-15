use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, sync::LazyLock};

pub const CONFIG: LazyLock<Config> = LazyLock::new(init_config);

fn init_config() -> Config {
    Figment::new()
        .merge(Toml::file("config.defaults.toml"))
        .merge(Toml::file("config.toml"))
        .extract()
        .unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub maps: MapConfig,
    pub game: GameConfig,
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

#[derive(Serialize, Deserialize)]
pub struct MapConfig {
    pub path: String,
    pub maps: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GameConfig {
    pub simulation_framerate: f32,
    pub spawn_map: Option<String>,
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

#[derive(Serialize, Deserialize)]
pub struct LoggerConfig {
    pub console: LoggerConsoleConfig,
    pub file: LoggerFileConfig,
    pub chat: LoggerChatConfig,
    pub panic_on_error: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LoggerConsoleConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub headers: Vec<LogHeaderType>,
    pub colored: bool,
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

#[derive(Serialize, Deserialize)]
pub struct LoggerChatConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub headers: Vec<LogHeaderType>,
}

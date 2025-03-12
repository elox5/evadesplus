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

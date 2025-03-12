use anyhow::Result;
use evadesplus::{
    cache::Cache,
    config::CONFIG,
    game::{game::Game, map_table::get_map_list},
    logger::Logger,
    networking::webtransport::WebTransportServer,
};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use tokio::sync::broadcast;
use warp::hyper::Uri;
use warp::Filter;
use wtransport::Identity;

#[tokio::main]
async fn main() -> Result<()> {
    let network_config = &CONFIG.network;

    if !std::path::Path::new(&network_config.client_path).is_dir() {
        panic!("Client code has not been compiled.");
    }

    let cache = Cache::new(get_map_list());
    let cache_hash = cache.get_hash();

    let (chat_tx, chat_rx) = broadcast::channel(8);

    let game = Game::new(chat_tx.clone());

    let identity =
        Identity::load_pemfiles(&network_config.ssl_cert_path, &network_config.ssl_key_path)
            .await
            .unwrap_or_else(|err| {
                Logger::warn(format!("Failed to load SSL certificate: {err}"));
                Logger::warn("Generating self-signed certificate... (browsers might react oddly)");

                Identity::self_signed([&network_config.ip.to_string()]).unwrap()
            });

    let cert = identity.certificate_chain().as_slice()[0].clone();
    let cert = cert.to_pem();

    let key = identity.private_key().clone_key();
    let key = key.to_secret_pem();

    let webtransport_server = WebTransportServer::new(
        identity,
        game,
        chat_tx,
        chat_rx,
        network_config.ip,
        network_config.client_port_https,
    )?;

    let root_route = warp::fs::dir(network_config.client_path.clone());
    let cache_route = warp::path("cache").and(warp::get()).then(move || {
        let cache = cache.clone();
        async move { warp::reply::json(&cache) }
    });
    let cache_hash_route = warp::path("cache_hash").and(warp::get()).then(move || {
        let hash = cache_hash.clone();
        async move { warp::reply::json(&hash) }
    });

    let routes = root_route.or(cache_route).or(cache_hash_route);
    let addr = webtransport_server.local_addr();

    let http_redirect_uri = Uri::from_str(&format!(
        "https://{}:{}",
        &network_config.ip, &network_config.client_port_https
    ))?;

    let http_route = warp::any().map(move || warp::redirect(http_redirect_uri.clone()));
    let http_addr = SocketAddr::new(
        IpAddr::V4(network_config.ip),
        network_config.client_port_http,
    );

    tokio::select! {
        _result = warp::serve(http_route).run(http_addr) => {
            Logger::info("HTTP server closed");
        }
        _result = warp::serve(routes).tls().cert(cert).key(key).run(addr) => {
            Logger::info("HTTPS server closed");
        }
        _result = webtransport_server.serve() => {
            Logger::info("WebTransport server closed");
        }
    }

    Ok(())
}

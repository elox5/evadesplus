use anyhow::Result;
use evadesplus::{
    cache::Cache,
    env::{get_env_or_default, get_env_var},
    game::{game::Game, map_table::get_map_list},
    networking::webtransport::WebTransportServer,
};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use warp::hyper::Uri;
use warp::Filter;
use wtransport::{tls::Sha256DigestFmt, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let local_ip_string = get_env_or_default("LOCAL_IP", "127.0.0.1");
    let local_ip = local_ip_string.parse().expect("Invalid local ip");

    let https_port = get_env_or_default("HTTPS_PORT", "443")
        .parse()
        .expect("Invalid port");
    let http_port: u16 = get_env_or_default("HTTP_PORT", "80")
        .parse()
        .expect("Invalid port");

    let client_path = get_env_or_default("CLIENT_PATH", "client/dist");

    if !std::path::Path::new(&client_path).is_dir() {
        panic!("Client code has not been compiled.");
    }

    let cache = Cache::new(get_map_list());
    let cache_hash = cache.get_hash();

    let start_map_id = get_env_var("START_MAP");

    let game = Game::new(start_map_id);

    let identity = Identity::self_signed([&local_ip_string])?;
    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    let cert = identity.certificate_chain().as_slice()[0].clone();
    let cert = cert.to_pem();

    let key = identity.private_key().clone_key();
    let key = key.to_secret_pem();

    let webtransport_server = WebTransportServer::new(identity, game, local_ip, https_port)?;

    let root_route = warp::fs::dir(client_path);
    let cert_route = warp::path("cert").and(warp::get()).then(move || {
        let cert_digest = cert_digest.clone();
        async move { warp::reply::json(&cert_digest.fmt(Sha256DigestFmt::BytesArray)) }
    });
    let cache_route = warp::path("cache").and(warp::get()).then(move || {
        let cache = cache.clone();
        async move { warp::reply::json(&cache) }
    });
    let cache_hash_route = warp::path("cache_hash").and(warp::get()).then(move || {
        let hash = cache_hash.clone();
        async move { warp::reply::json(&hash) }
    });

    let routes = root_route
        .or(cert_route)
        .or(cache_route)
        .or(cache_hash_route);
    let addr = webtransport_server.local_addr();

    let http_redirect_uri =
        Uri::from_str(&format!("https://{}:{}", &local_ip_string, &https_port))?;

    let http_route = warp::any().map(move || warp::redirect(http_redirect_uri.clone()));
    let http_addr = SocketAddr::new(IpAddr::V4(local_ip), http_port);

    tokio::select! {
        _result = warp::serve(http_route).run(http_addr) => {
            println!("HTTP server closed");
        }
        _result = warp::serve(routes).tls().cert(cert).key(key).run(addr) => {
            println!("HTTPS server closed");
        }
        _result = webtransport_server.serve() => {
            println!("WebTransport server closed");
        }
    }

    Ok(())
}

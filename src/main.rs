use anyhow::Result;
use evadesplus::{
    cache::Cache,
    env::{get_env_or_default, get_env_var, try_get_env_var},
    game::game::Game,
    networking::webtransport::WebTransportServer,
    parsing::parse_map,
};
use std::{
    ffi::OsStr,
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

    let map_path = get_env_or_default("MAP_PATH", "maps");

    let maps = try_get_env_var("MAPS");

    let start_area_id = get_env_var("START_AREA_ID");

    let maps = match maps {
        Some(m) => m
            .split(',')
            .into_iter()
            .map(|m| parse_map(&format!("{}/{}.yaml", map_path, m)).unwrap())
            .collect::<Vec<_>>(),
        None => std::fs::read_dir(map_path)
            .unwrap()
            .filter_map(|f| f.ok())
            .filter(|f| f.path().is_file())
            .filter(|f| f.path().extension().unwrap_or(OsStr::new("")) == "yaml")
            .map(|f| parse_map(f.path().to_str().unwrap()).unwrap())
            .collect::<Vec<_>>(),
    };

    let cache = Cache::new(&maps);

    let game = Game::new(maps, &start_area_id);

    let identity = Identity::self_signed([&local_ip_string])?;
    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    let cert = identity.certificate_chain().as_slice()[0].clone();
    let cert = cert.to_pem();

    let key = identity.private_key().clone_key();
    let key = key.to_secret_pem();

    let webtransport_server = WebTransportServer::new(identity, game, local_ip, https_port)?;

    let root_route = warp::fs::dir("static");
    let cert_route = warp::path("cert").and(warp::get()).then(move || {
        let cert_digest = cert_digest.clone();
        async move { warp::reply::json(&cert_digest.fmt(Sha256DigestFmt::BytesArray)) }
    });
    let cache_route = warp::path("cache").and(warp::get()).then(move || {
        let cache = cache.clone();
        async move { warp::reply::json(&cache) }
    });

    let routes = root_route.or(cert_route).or(cache_route);
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

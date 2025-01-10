use anyhow::Result;
use evadesplus::{
    game::game::Game, networking::webtransport::WebTransportServer, parsing::parse_map,
};
use warp::Filter;
use wtransport::{tls::Sha256DigestFmt, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let env_local_ip = std::env::var("LOCAL_IP").expect(".env LOCAL_IP must be set");
    let env_port = std::env::var("PORT").expect(".env PORT must be set");
    let map_path = std::env::var("MAP_PATH").expect(".env MAP_PATH must be set");
    let maps = std::env::var("MAPS").expect(".env MAPS must be set");
    let maps = maps.split(',').collect::<Vec<_>>();
    let start_area_id = std::env::var("START_AREA_ID").expect(".env START_AREA_ID must be set");

    if maps.len() == 0 {
        panic!(".env MAPS must contain at least one map");
    }

    let maps = maps
        .iter()
        .map(|m| parse_map(&format!("{}/{}.yaml", map_path, m)).unwrap())
        .collect::<Vec<_>>();

    let local_ip = env_local_ip.parse().expect("Invalid local ip");
    let port = env_port.parse().expect("Invalid port");

    let game = Game::new(maps, &start_area_id);

    let identity = Identity::self_signed([env_local_ip])?;
    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    let cert = identity.certificate_chain().as_slice()[0].clone();
    let cert = cert.to_pem();

    let key = identity.private_key().clone_key();
    let key = key.to_secret_pem();

    let webtransport_server = WebTransportServer::new(identity, game, local_ip, port)?;

    let root_route = warp::fs::dir("static");
    let cert_route = warp::path("cert").and(warp::get()).then(move || {
        let cert_digest = cert_digest.clone();
        async move { warp::reply::json(&cert_digest.fmt(Sha256DigestFmt::BytesArray)) }
    });

    let routes = root_route.or(cert_route);
    let addr = webtransport_server.local_addr();

    tokio::select! {
        _result = warp::serve(routes).tls().cert(cert).key(key).run(addr) => {
            println!("HTTPS server closed");
        }
        _result = webtransport_server.serve() => {
            println!("WebTransport server closed");
        }
    }

    Ok(())
}

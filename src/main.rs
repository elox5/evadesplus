use anyhow::Result;
use evadesplus::{
    env::get_env_var, game::game::Game, networking::webtransport::WebTransportServer,
    parsing::parse_map,
};
use warp::Filter;
use wtransport::{tls::Sha256DigestFmt, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let local_ip_string = get_env_var("LOCAL_IP");
    let local_ip = local_ip_string.parse().expect("Invalid local ip");

    let port = get_env_var("PORT").parse().expect("Invalid port");

    let map_path = get_env_var("MAP_PATH");

    let maps = get_env_var("MAPS");
    let maps = maps.split(',').collect::<Vec<_>>();

    let start_area_id = get_env_var("START_AREA_ID");

    if maps.len() == 0 {
        panic!(".env MAPS must contain at least one map");
    }

    let maps = maps
        .iter()
        .map(|m| parse_map(&format!("{}/{}.yaml", map_path, m)).unwrap())
        .collect::<Vec<_>>();

    let game = Game::new(maps, &start_area_id);

    let identity = Identity::self_signed([local_ip_string])?;
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

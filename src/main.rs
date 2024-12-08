use anyhow::Result;
use evadesplus::{game::area::Area, networking::webtransport::WebTransportServer};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use wtransport::{tls::Sha256DigestFmt, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    let area = Area::new("test".to_string(), "test".to_string());
    let area_arc = Arc::new(Mutex::new(area));

    Area::start_update_loop(area_arc.clone());

    let identity = Identity::self_signed(["localhost", "127.0.0.1", "[::1]"])?;
    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    let webtransport_server = WebTransportServer::new(identity, area_arc.clone())?;

    let root_route = warp::fs::dir("static");
    let cert_route = warp::path("cert").and(warp::get()).then(move || {
        let cert_digest = cert_digest.clone();
        async move { warp::reply::json(&cert_digest.fmt(Sha256DigestFmt::BytesArray)) }
    });

    let routes = root_route.or(cert_route);
    let addr = webtransport_server.local_addr();

    tokio::select! {
        _result = warp::serve(routes).run(addr) => {
            println!("HTTP server closed");
        }
        _result = webtransport_server.serve() => {
            println!("WebTransport server closed");
        }
    }

    Ok(())
}

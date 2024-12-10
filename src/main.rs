use anyhow::Result;
use evadesplus::{
    game::{
        area::Area,
        components::{
            BounceOffBounds, Bounded, Color, Direction, Enemy, Position, Size, Speed, Velocity,
        },
    },
    networking::webtransport::WebTransportServer,
    physics::vec2::Vec2,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use wtransport::{tls::Sha256DigestFmt, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    let mut area = Area::new(
        "test".to_string(),
        "Testing Territory".to_string(),
        100.0,
        100.0,
    );

    area.world.spawn_batch((0..1000).map(|_| {
        let pos = Position(area.bounds.random_inside());
        let vel = Velocity(Vec2::ZERO);
        let dir = Direction(Vec2::random_unit());
        let speed = Speed(10.0);
        let size = Size(1.0);
        let color = Color::rgb(100, 100, 100);

        (
            Enemy,
            pos,
            vel,
            dir,
            speed,
            size,
            color,
            Bounded,
            BounceOffBounds,
        )
    }));

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

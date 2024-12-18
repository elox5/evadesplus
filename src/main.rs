use anyhow::Result;
use evadesplus::{
    game::{
        components::Color,
        game::Game,
        templates::{AreaTemplate, EnemyGroup, MapTemplate},
    },
    networking::webtransport::WebTransportServer,
    physics::rect::Rect,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use wtransport::{tls::Sha256DigestFmt, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    let mut game = Game::new();

    let map = MapTemplate::new(
        "tt".to_owned(),
        "Testing Territory".to_owned(),
        Color::rgb(200, 200, 200),
        vec![
            AreaTemplate::new(
                None,
                None,
                100.0,
                15.0,
                vec![
                    Rect::new(40.0, 5.0, 7.0, 5.0),
                    Rect::new(30.0, 3.0, 10.0, 2.0),
                ],
                vec![
                    EnemyGroup::new(Color::rgb(100, 100, 100), 50, 5.0, 1.0),
                    EnemyGroup::new(Color::rgb(0, 0, 0), 50, 10.0, 0.3),
                ],
            ),
            AreaTemplate::new(
                Some("Named Area".to_owned()),
                Some(Color::rgb(100, 200, 100)),
                500.0,
                15.0,
                Vec::new(),
                vec![
                    EnemyGroup::new(Color::rgb(200, 200, 200), 10, 5.0, 3.0),
                    EnemyGroup::new(Color::rgb(255, 0, 0), 100, 1.0, 0.3),
                ],
            ),
        ],
    );

    game.try_create_area(&map, 1);

    let game_arc = Arc::new(Mutex::new(game));

    let identity = Identity::self_signed(["localhost", "127.0.0.1", "[::1]"])?;
    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    let webtransport_server = WebTransportServer::new(identity, game_arc)?;

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

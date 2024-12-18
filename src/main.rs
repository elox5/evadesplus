use anyhow::Result;
use evadesplus::{
    game::{
        components::Color,
        data::{AreaData, EnemyGroup, MapData},
        game::Game,
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
    let map = MapData {
        id: "tt".to_owned(),
        name: "Testing Territory".to_owned(),
        background_color: Color::rgb(200, 200, 200),
        areas: vec![
            AreaData {
                id: None,
                name: None,
                background_color: None,
                width: 100.0,
                height: 15.0,
                inner_walls: vec![
                    Rect::new(40.0, 5.0, 7.0, 5.0),
                    Rect::new(30.0, 3.0, 10.0, 2.0),
                ],
                enemy_groups: vec![
                    EnemyGroup::new(Color::rgb(100, 100, 100), 50, 5.0, 1.0),
                    EnemyGroup::new(Color::rgb(0, 0, 0), 50, 10.0, 0.3),
                ],
            },
            AreaData {
                id: None,
                name: Some("Named Area".to_owned()),
                background_color: Some(Color::rgb(100, 200, 100)),
                width: 500.0,
                height: 15.0,
                inner_walls: Vec::new(),
                enemy_groups: vec![
                    EnemyGroup::new(Color::rgb(200, 200, 200), 10, 5.0, 3.0),
                    EnemyGroup::new(Color::rgb(255, 0, 0), 100, 1.0, 0.3),
                ],
            },
        ],
    };

    let mut game = Game::new(vec![map]);

    let _ = game.try_create_area("tt:0");

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

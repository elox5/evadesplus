use crate::{game::game::Game, physics::vec2::Vec2};
use anyhow::Result;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use wtransport::{
    endpoint::{endpoint_side::Server, IncomingSession},
    Endpoint, Identity, ServerConfig,
};

pub struct WebTransportServer {
    endpoint: Endpoint<Server>,
    game: Arc<Mutex<Game>>,
}

impl WebTransportServer {
    pub fn new(identity: Identity, game: Arc<Mutex<Game>>) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_bind_address(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                3333,
            ))
            .with_identity(identity)
            .keep_alive_interval(Some(Duration::from_secs(10)))
            .build();

        let endpoint = Endpoint::server(config)?;

        Ok(Self { endpoint, game })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.endpoint.local_addr().unwrap()
    }

    pub async fn serve(self) -> Result<()> {
        println!("WebTransport server listening on {}", &self.local_addr());

        for id in 0.. {
            let incomming_session = self.endpoint.accept().await;

            println!("Accepting session {}", incomming_session.remote_address());

            tokio::spawn(Self::handle_session(
                incomming_session,
                self.game.clone(),
                id,
            ));
        }

        Ok(())
    }

    async fn handle_session(session: IncomingSession, game: Arc<Mutex<Game>>, id: u64) {
        let result = Self::handle_session_impl(session, game, id).await;

        println!("Session {id} closed with result: {result:?}");
    }

    async fn handle_session_impl(
        session: IncomingSession,
        game: Arc<Mutex<Game>>,
        id: u64,
    ) -> Result<()> {
        let mut buffer = vec![0; 65536].into_boxed_slice();

        let session_request = session.await?;

        println!(
            "New session request from client {id}: Authority: '{}', Path: '{}'",
            session_request.authority(),
            session_request.path()
        );

        let connection = session_request.accept().await?;

        println!("Accepted connection from client {id}. Awaiting streams...");

        let mut entity = None;

        loop {
            tokio::select! {
                stream = connection.accept_uni() => {
                    let mut stream = stream?;

                    let bytes_read = match stream.read(&mut buffer).await? {
                        Some(bytes_read) => bytes_read,
                        None => continue,
                    };

                    let data = &buffer[..bytes_read];
                    let name = std::str::from_utf8(data);


                    if let Ok(name) = name {
                        println!("Accepted name '{name}' from client {id}. Spawning hero...");

                        let game = game.lock().await;
                        let mut area = game.areas[0].lock().await;

                        entity = Some(area.spawn_hero(name, connection.clone()));

                        let definition = area.definition_packet();

                        let mut response_stream = connection.open_uni().await?.await?;
                        response_stream.write_all(&definition).await?;
                        response_stream.finish().await?;
                    }
                }
                dgram = connection.receive_datagram() => {
                    if let Some(entity) = entity {
                        let dgram = dgram?;
                        let payload = dgram.payload();

                        let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                        let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);

                        // println!("Received input '({x:.2}, {y:.2})' from client {id}");

                        let  game = game.lock().await;
                        let mut area = game.areas[0].lock().await;
                        area.update_hero_dir(entity, Vec2::new(x, y));
                    }
                }
                _ = connection.closed() => {
                    println!("Connection from client {id} closed");

                    if let Some(entity) = entity {
                        let game = game.lock().await;
                        let mut area = game.areas[0].lock().await;

                        let _ = area.world.despawn(entity);
                    }

                    return Ok(());
                }
            }
        }
    }
}

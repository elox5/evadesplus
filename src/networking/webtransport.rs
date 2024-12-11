use crate::{game::area::Area, physics::vec2::Vec2};
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
    area_arc: Arc<Mutex<Area>>,
}

impl WebTransportServer {
    pub fn new(identity: Identity, area_arc: Arc<Mutex<Area>>) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_bind_address(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                3333,
            ))
            .with_identity(identity)
            .keep_alive_interval(Some(Duration::from_secs(10)))
            .build();

        let endpoint = Endpoint::server(config)?;

        Ok(Self { endpoint, area_arc })
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
                self.area_arc.clone(),
                id,
            ));
        }

        Ok(())
    }

    async fn handle_session(session: IncomingSession, area_arc: Arc<Mutex<Area>>, id: u64) {
        let result = Self::handle_session_impl(session, area_arc, id).await;

        println!("Session {id} closed with result: {result:?}");
    }

    async fn handle_session_impl(
        session: IncomingSession,
        area_arc: Arc<Mutex<Area>>,
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

                        let mut area = area_arc.lock().await;
                        entity = Some(area.spawn_hero(name, connection.clone()));

                        let mut response = Vec::<u8>::new();
                        response.extend_from_slice(b"ADEF"); // area definition
                        response.extend_from_slice(&area.bounds.w.to_le_bytes());
                        response.extend_from_slice(&area.bounds.h.to_le_bytes());
                        response.extend_from_slice(&area.background_color.to_bytes());

                        response.extend_from_slice(&(area.inner_walls.len() as u16).to_le_bytes());

                        for wall in &area.inner_walls {
                            response.extend_from_slice(&wall.to_bytes());
                        }

                        response.extend_from_slice(&area.name.len().to_le_bytes()[..4]);
                        response.extend_from_slice(area.name.as_bytes());

                        let mut response_stream = connection.open_uni().await?.await?;
                        response_stream.write_all(&response).await?;
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

                        let mut area = area_arc.lock().await;
                        area.update_hero_dir(entity, Vec2::new(x, y));
                    }
                }
                _ = connection.closed() => {
                    println!("Connection from client {id} closed");

                    if let Some(entity) = entity {
                        let mut area = area_arc.lock().await;
                        let _ = area.world.despawn(entity);
                    }

                    return Ok(());
                }
            }
        }
    }
}

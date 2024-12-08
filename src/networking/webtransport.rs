use crate::game::game::World;
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
    world_arc: Arc<Mutex<World>>,
}

impl WebTransportServer {
    pub fn new(identity: Identity, world_arc: Arc<Mutex<World>>) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_bind_address(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                3333,
            ))
            .with_identity(identity)
            .keep_alive_interval(Some(Duration::from_secs(10)))
            .build();

        let endpoint = Endpoint::server(config)?;

        Ok(Self {
            endpoint,
            world_arc,
        })
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
                self.world_arc.clone(),
                id,
            ));
        }

        Ok(())
    }

    async fn handle_session(session: IncomingSession, world_arc: Arc<Mutex<World>>, id: u64) {
        let result = Self::handle_session_impl(session, world_arc, id).await;

        println!("Session {id} closed with result: {result:?}");
    }

    async fn handle_session_impl(
        session: IncomingSession,
        world_arc: Arc<Mutex<World>>,
        id: u64,
    ) -> Result<()> {
        let mut _buffer = vec![0; 65536].into_boxed_slice();

        let session_request = session.await?;

        println!(
            "New session request from client {id}: Authority: '{}', Path: '{}'",
            session_request.authority(),
            session_request.path()
        );

        let connection = session_request.accept().await?;

        {
            let mut world = world_arc.lock().await;
            world.create_player(id, connection.clone());
        }

        println!("Accepted connection from client {id}. Awaiting streams...");

        loop {
            tokio::select! {
                _stream = connection.accept_bi() => {
                    println!("Accepted bidirectional stream from client {id}");
                },
                _stream = connection.accept_uni() => {
                    println!("Accepted unidirectional stream from client {id}");
                },
                dgram = connection.receive_datagram() => {
                    let dgram = dgram?;
                    let payload = dgram.payload();

                    let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);

                    println!("Received input '({x:.2}, {y:.2})' from client {id}");

                    let mut world = world_arc.lock().await;
                    world.update_player_input(id, x, y);
                }
            }
        }
    }
}

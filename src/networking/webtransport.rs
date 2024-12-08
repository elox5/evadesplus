use crate::game::game::World;
use anyhow::Result;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use wtransport::{
    datagram::Datagram,
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

        loop {
            let incomming_session = self.endpoint.accept().await;

            println!("Accepting session {}", incomming_session.remote_address());

            let mut world = self.world_arc.lock().await;

            let player = world.create_player();

            let id = player.id;

            tokio::spawn(Self::handle_session(incomming_session, id));
        }
    }

    async fn handle_session(session: IncomingSession, id: u64) {
        let result = Self::handle_session_impl(session, id).await;

        println!("Session {:X} closed with result: {:?}", id, result);
    }

    async fn handle_session_impl(session: IncomingSession, id: u64) -> Result<()> {
        let mut buffer = vec![0; 65536].into_boxed_slice();

        let session_request = session.await?;

        println!(
            "New session request from client {id:X}: Authority: '{}', Path: '{}'",
            session_request.authority(),
            session_request.path()
        );

        let connection = session_request.accept().await?;

        println!("Accepted connection from client {id:X}. Awaiting streams...");

        loop {
            tokio::select! {
                stream = connection.accept_bi() => {
                    let mut stream = stream?;
                    println!("Accepted bidirectional stream");

                    let bytes_read = match stream.1.read(&mut buffer).await? {
                        Some(bytes_read) => bytes_read,
                        None => continue,
                    };

                    let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                    println!("Received (bi) '{str_data}' from client {id:X}");

                    stream.0.write_all(b"ACK").await?;
                },
                stream = connection.accept_uni() => {
                    let mut stream = stream?;
                    println!("Accepted unidirectional stream from client {id:X}");

                    let bytes_read = match stream.read(&mut buffer).await? {
                        Some(bytes_read) => bytes_read,
                        None => continue,
                    };

                    let data = &buffer[..bytes_read];

                    let x = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    let y = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);

                    println!("Received (uni) '({x:.2}, {y:.2})' from client {id:X}");
                },
                dgram = connection.receive_datagram() => {
                    let dgram = dgram?;
                    let payload = dgram.payload();

                    let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);

                    println!("Received (dgram) '({x:.2}, {y:.2})' from client {id:X}");

                    connection.send_datagram(b"ACK")?;
                }
            }
        }
    }
}

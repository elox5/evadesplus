use anyhow::Result;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use wtransport::{
    endpoint::{endpoint_side::Server, IncomingSession},
    Endpoint, Identity, ServerConfig,
};

pub struct WebTransportServer {
    endpoint: Endpoint<Server>,
}

impl WebTransportServer {
    pub fn new(identity: Identity) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_bind_address(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                3333,
            ))
            .with_identity(identity)
            .keep_alive_interval(Some(Duration::from_secs(10)))
            .build();

        let endpoint = Endpoint::server(config)?;

        Ok(Self { endpoint })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.endpoint.local_addr().unwrap()
    }

    pub async fn serve(self) -> Result<()> {
        println!("WebTransport server listening on {}", &self.local_addr());

        for id in 0.. {
            let incomming_session = self.endpoint.accept().await;

            println!(
                "Accepting session {} (id: {})",
                incomming_session.remote_address(),
                id
            );

            tokio::spawn(Self::handle_session(incomming_session, id));
        }

        Ok(())
    }

    async fn handle_session(session: IncomingSession, id: u32) {
        let result = Self::handle_session_impl(session, id).await;

        println!("Session {} closed with result: {:?}", id, result);
    }

    async fn handle_session_impl(session: IncomingSession, id: u32) -> Result<()> {
        let mut buffer = vec![0; 65536].into_boxed_slice();

        let session_request = session.await?;

        println!(
            "New session request from client {id}: Authority: '{}', Path: '{}'",
            session_request.authority(),
            session_request.path()
        );

        let connection = session_request.accept().await?;

        println!("Accepted connection from client {id}. Awaiting streams...");

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

                    println!("Received (bi) '{str_data}' from client");

                    stream.0.write_all(b"ACK").await?;
                },
                stream = connection.accept_uni() => {
                    let mut stream = stream?;
                    println!("Accepted unidirectional stream from client {id}");

                    let bytes_read = match stream.read(&mut buffer).await? {
                        Some(bytes_read) => bytes_read,
                        None => continue,
                    };

                    let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                    println!("Received (uni) '{str_data}' from client {id}");

                    let mut stream = connection.open_uni().await?.await?;
                    stream.write_all(b"ACK").await?;
                },
                dgram = connection.receive_datagram() => {
                    let dgram = dgram?;
                    let str_data = std::str::from_utf8(&dgram)?;

                    println!("Received dgram '{str_data}' from client {id}");

                    connection.send_datagram(b"ACK")?;
                }
            }
        }
    }
}

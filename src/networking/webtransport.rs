use super::{
    chat::{ChatMessageType, ChatRequest},
    commands::{handle_command, CommandRequest},
    leaderboard::LeaderboardUpdatePacket,
};
use crate::{
    game::game::{Game, Player},
    physics::vec2::Vec2,
};
use anyhow::Result;
use arc_swap::ArcSwap;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{broadcast, Mutex};
use wtransport::{
    datagram::Datagram,
    endpoint::{endpoint_side::Server, IncomingSession},
    error::ConnectionError,
    Connection, Endpoint, Identity, RecvStream, SendStream, ServerConfig,
};

pub struct WebTransportServer {
    endpoint: Endpoint<Server>,
    game: Arc<Mutex<Game>>,
    chat_tx: broadcast::Sender<ChatRequest>,
    chat_rx: broadcast::Receiver<ChatRequest>,
}

impl WebTransportServer {
    pub fn new(
        identity: Identity,
        game: Arc<Mutex<Game>>,
        local_ip: Ipv4Addr,
        port: u16,
    ) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_bind_address(SocketAddr::new(IpAddr::V4(local_ip), port))
            .with_identity(identity)
            .keep_alive_interval(Some(Duration::from_secs(10)))
            .build();

        let endpoint = Endpoint::server(config)?;

        let (chat_tx, chat_rx) = broadcast::channel(16);

        Ok(Self {
            endpoint,
            game,
            chat_tx,
            chat_rx,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.endpoint.local_addr().unwrap()
    }

    pub async fn serve(self) -> Result<()> {
        println!("WebTransport server listening on {}", &self.local_addr());

        for id in 0.. {
            let incomming_session = self.endpoint.accept().await;

            println!(
                "Accepting session {id} from {}",
                incomming_session.remote_address()
            );

            tokio::spawn(Self::handle_session(
                incomming_session,
                self.game.clone(),
                self.chat_tx.clone(),
                self.chat_rx.resubscribe(),
                id,
            ));
        }

        Ok(())
    }

    async fn handle_session(
        session: IncomingSession,
        game: Arc<Mutex<Game>>,
        chat_tx: broadcast::Sender<ChatRequest>,
        chat_rx: broadcast::Receiver<ChatRequest>,
        id: u64,
    ) {
        let result = Self::handle_session_impl(session, game, chat_tx, chat_rx, id).await;

        println!("Session {id} closed with result: {result:?}");
    }

    async fn handle_session_impl(
        session: IncomingSession,
        game: Arc<Mutex<Game>>,
        chat_tx: broadcast::Sender<ChatRequest>,
        mut chat_rx: broadcast::Receiver<ChatRequest>,
        id: u64,
    ) -> Result<ConnectionError> {
        let mut buffer = vec![0; 65536].into_boxed_slice();

        let session_request = session.await?;

        let connection = session_request.accept().await?;

        println!("Accepted connection from client {id}. Awaiting streams...");

        let mut player: Option<Arc<ArcSwap<Player>>> = None;

        let mut lb_rx = game.lock().await.leaderboard_rx.resubscribe();

        loop {
            tokio::select! {
                stream = connection.accept_uni() => {
                    let stream = stream?;
                    Self::handle_uni_stream(stream, &mut buffer, &connection, &game, id, &mut player, &chat_tx).await?;
                }
                streams = connection.accept_bi() => {
                    let streams = streams?;
                    let (send_stream, recv_stream) = streams;
                    Self::handle_bi_stream(send_stream, recv_stream, &mut buffer).await?;
                }
                dgram = connection.receive_datagram() => {
                    let dgram = dgram?;
                    Self::handle_datagram(dgram, &game, &player).await;
                }
                connection_result = connection.closed() => {
                    return Self::handle_connection_closed(connection_result, &game, &player, id).await;
                }
                leaderboard_update = lb_rx.recv() => {
                    let leaderboard_update = leaderboard_update?;
                    Self::handle_leaderboard_update(leaderboard_update, &connection).await?;
                }
                chat_broadcast = chat_rx.recv() => {
                    let chat_broadcast = chat_broadcast?;
                    Self::handle_chat_broadcast(chat_broadcast, &connection, id).await?;
                }
            }
        }
    }

    async fn handle_uni_stream(
        mut stream: RecvStream,
        buffer: &mut Box<[u8]>,
        connection: &Connection,
        game: &Arc<Mutex<Game>>,
        id: u64,
        player: &mut Option<Arc<ArcSwap<Player>>>,
        chat_tx: &broadcast::Sender<ChatRequest>,
    ) -> Result<()> {
        let bytes_read = match stream.read(buffer).await? {
            Some(bytes_read) => bytes_read,
            None => return Ok(()),
        };

        let data = &buffer[..bytes_read];
        let header = &data[0..4];
        let data = &data[4..];

        match header {
            b"NAME" => {
                let name = std::str::from_utf8(data)?;

                println!("Accepted name '{name}' from client {id}. Spawning hero...");

                let mut game = game.lock().await;
                let leaderboard_state = game.leaderboard_state.clone();

                let player_arcswap = game.spawn_hero(id, name, connection.clone()).await;
                *player = Some(player_arcswap.clone());

                let area = player_arcswap.load().area.clone();

                let definition = area.lock().await.definition_packet();

                if !leaderboard_state.is_empty() {
                    let mut state_stream = connection.open_uni().await?.await?;
                    state_stream
                        .write_all(&leaderboard_state.to_bytes())
                        .await?;
                    state_stream.finish().await?;
                }

                let mut def_stream = connection.open_uni().await?.await?;
                def_stream.write_all(&definition).await?;
                def_stream.finish().await?;
            }
            b"CHAT" => {
                if let Some(player) = &player {
                    let text = std::str::from_utf8(data)?;

                    if text.starts_with("/") {
                        let text = &text[1..];
                        let splits = text.split(" ").collect::<Vec<&str>>();
                        let command = splits[0];
                        let args = splits[1..].iter().map(|s| s.to_string()).collect();

                        let req = CommandRequest::new(args, game.clone(), player.clone());

                        let response = handle_command(command, req).await;

                        if let Some(response) = response {
                            let _ = chat_tx.send(response);
                        }
                    } else {
                        let name = player.load().name.clone();

                        let request = ChatRequest::new(
                            text.to_owned(),
                            name.clone(),
                            ChatMessageType::Normal,
                            None,
                        );

                        let _ = chat_tx.send(request);
                    }
                }
            }
            _ => {
                println!(
                    "Received unknown packet from client {id} (header: {})",
                    std::str::from_utf8(header).unwrap_or(&format!("{header:x?}").clone())
                );
            }
        }

        Ok(())
    }

    async fn handle_bi_stream(
        mut send_stream: SendStream,
        mut recv_stream: RecvStream,
        buffer: &mut Box<[u8]>,
    ) -> Result<()> {
        let bytes_read = match recv_stream.read(buffer).await? {
            Some(bytes_read) => bytes_read,
            None => return Ok(()),
        };

        let data = &buffer[..bytes_read];
        let text = std::str::from_utf8(data)?;

        if text == "ping" {
            send_stream.write_all(b"pong").await?;
        }

        send_stream.finish().await?;

        Ok(())
    }

    async fn handle_datagram(
        datagram: Datagram,
        game: &Arc<Mutex<Game>>,
        player: &Option<Arc<ArcSwap<Player>>>,
    ) {
        if let Some(player) = player {
            let payload = datagram.payload();

            let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
            let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);

            // println!("Received input '({x:.2}, {y:.2})' from client {id}");

            let mut game = game.lock().await;
            game.update_player_input(player.load().entity, Vec2::new(x, y))
                .await;
        }
    }

    async fn handle_connection_closed(
        connection_result: ConnectionError,
        game: &Arc<Mutex<Game>>,
        player: &Option<Arc<ArcSwap<Player>>>,
        id: u64,
    ) -> Result<ConnectionError> {
        println!("Connection from client {id} closed");

        if let Some(player) = player {
            let mut game = game.lock().await;
            game.despawn_hero(player).await;
        }

        Ok(connection_result)
    }

    async fn handle_leaderboard_update(
        update: LeaderboardUpdatePacket,
        connection: &Connection,
    ) -> Result<()> {
        let mut update_stream = connection.open_uni().await?.await?;

        update_stream.write_all(&update.to_bytes()).await?;
        update_stream.finish().await?;

        Ok(())
    }

    async fn handle_chat_broadcast(
        request: ChatRequest,
        connection: &Connection,
        id: u64,
    ) -> Result<()> {
        if let Some(ref filter) = request.recipient_filter {
            if !filter.contains(&id) {
                return Ok(());
            }
        }

        let mut update_stream = connection.open_uni().await?.await?;

        update_stream.write_all(&request.to_bytes()).await?;
        update_stream.finish().await?;

        Ok(())
    }
}

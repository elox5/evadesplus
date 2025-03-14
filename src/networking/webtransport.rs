use super::{
    chat::{Chat, ChatMessageType, ChatRequest},
    commands::{handle_command, CommandRequest},
    leaderboard::LeaderboardUpdate,
};
use crate::{
    game::{area::Area, game::Game},
    logger::{LogCategory, Logger},
    physics::vec2::Vec2,
};
use anyhow::Result;
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
        game_arc: Arc<Mutex<Game>>,
        host_ip: Ipv4Addr,
        port: u16,
    ) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_bind_address(SocketAddr::new(IpAddr::V4(host_ip), port))
            .with_identity(identity)
            .keep_alive_interval(Some(Duration::from_secs(10)))
            .build();

        let endpoint = Endpoint::server(config)?;

        let game = game_arc.try_lock().unwrap();
        drop(game);

        Ok(Self {
            endpoint,
            game: game_arc,
            chat_tx: Chat::tx(),
            chat_rx: Chat::rx(),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.endpoint.local_addr().unwrap()
    }

    pub async fn serve(self) -> Result<()> {
        Logger::info(format!(
            "WebTransport server listening on https://{}",
            &self.local_addr()
        ));

        for id in 0.. {
            let incomming_session = self.endpoint.accept().await;

            Logger::log(
                format!(
                    "Accepting session @{id} from {}",
                    incomming_session.remote_address()
                ),
                LogCategory::Network,
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
        let result = Self::handle_session_impl(session, game.clone(), chat_tx, chat_rx, id).await;

        Logger::log(
            format!("Session @{id} closed with result: {result:?}"),
            LogCategory::Network,
        );

        Self::finalize_connection(&game, id).await;
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

        Logger::log(
            format!("Accepted connection from client @{id}. Awaiting streams..."),
            LogCategory::Network,
        );

        let mut lb_rx = game.lock().await.leaderboard_rx.resubscribe();

        loop {
            tokio::select! {
                stream = connection.accept_uni() => {
                    let stream = stream?;
                    handle_uni_stream(stream, &mut buffer, &connection, &game, id, &chat_tx).await?;
                }
                streams = connection.accept_bi() => {
                    let streams = streams?;
                    let (send_stream, recv_stream) = streams;
                    handle_bi_stream(send_stream, recv_stream, &mut buffer, &connection, &game, id).await?;
                }
                dgram = connection.receive_datagram() => {
                    let dgram = dgram?;
                    handle_datagram(dgram, &game, id).await;
                }
                connection_result = connection.closed() => {
                    return Ok(connection_result);
                }
                leaderboard_update = lb_rx.recv() => {
                    let leaderboard_update = leaderboard_update?;
                    handle_leaderboard_update(leaderboard_update, &connection).await?;
                }
                chat_broadcast = chat_rx.recv() => {
                    let chat_broadcast = chat_broadcast?;
                    handle_chat_broadcast(chat_broadcast, &connection, id).await?;
                }
            }
        }
    }

    async fn finalize_connection(game: &Arc<Mutex<Game>>, id: u64) {
        let mut game = game.lock().await;
        let _ = game.despawn_hero(id).await;
    }
}

async fn handle_uni_stream(
    mut stream: RecvStream,
    buffer: &mut Box<[u8]>,
    connection: &Connection,
    game: &Arc<Mutex<Game>>,
    id: u64,
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
        b"CHAT" => {
            let text = std::str::from_utf8(data)?;

            Logger::log(format!("[@{id}] {text}"), LogCategory::Chat);

            if text.starts_with("/") {
                let text = &text[1..];
                let splits = text.split(" ").collect::<Vec<&str>>();
                let command = splits[0];
                let args = splits[1..].iter().map(|s| s.to_string()).collect();

                let req = CommandRequest {
                    args,
                    game: game.clone(),
                    player_id: id,
                };

                Logger::info(format!(
                    "Executing command '{command}' requested by client @{id}..."
                ));

                let response = handle_command(command, req).await;

                let message = match response {
                        Ok(response) => response,
                        Err(err) => Some(ChatRequest::new(
                            format!("A server error has occurred. Please report it to the developers: *{err:?}*"),
                            String::new(),
                            id,
                            ChatMessageType::ServerError,
                            Some(vec![id]),
                        )),
                    };

                if let Some(message) = message {
                    if message.recipient_filter == Some(vec![id]) {
                        let _ = send_chat_message(message, connection).await;
                    } else {
                        let _ = chat_tx.send(message);
                    }
                }
            } else {
                let game = game.lock().await;
                let name = game.get_player(id)?.name.clone();

                let request =
                    ChatRequest::new(text.to_owned(), name, id, ChatMessageType::Normal, None);

                let _ = chat_tx.send(request);
            }
        }
        _ => handle_unknown_header(header, id),
    }

    Ok(())
}

async fn handle_bi_stream(
    mut send_stream: SendStream,
    mut recv_stream: RecvStream,
    buffer: &mut Box<[u8]>,
    connection: &Connection,
    game: &Arc<Mutex<Game>>,
    id: u64,
) -> Result<()> {
    let bytes_read = match recv_stream.read(buffer).await? {
        Some(bytes_read) => bytes_read,
        None => return Ok(()),
    };

    let data = &buffer[..bytes_read];
    let header = &data[0..4];
    let data = &data[4..];

    match header {
        b"PING" => {
            send_stream.write_all(b"PONG").await?;
        }
        b"INIT" => {
            let mut response: Vec<u8> = Vec::new();

            let name = std::str::from_utf8(data)?;

            let valid = validate_player_name(name);

            if valid {
                Logger::log(
                    format!("Accepted name '{name}' from client @{id}. Spawning hero..."),
                    LogCategory::Network,
                );

                let spawn_result = spawn_hero(name, connection, game, id).await;

                match spawn_result {
                    Ok(res) => {
                        response.push(0);
                        response.extend_from_slice(&res);
                    }
                    Err(err) => {
                        response.push(2);

                        let msg = err.to_string();
                        let length = msg.len() as u16;

                        response.extend_from_slice(&length.to_le_bytes());
                        response.extend_from_slice(msg.as_bytes());
                    }
                }
            } else {
                Logger::log(
                    format!("Rejected client @{id} for invalid name '{name}'"),
                    LogCategory::Network,
                );
                response.push(1);
            }

            send_stream.write_all(&response).await?;
        }
        _ => handle_unknown_header(header, id),
    }

    send_stream.finish().await?;

    Ok(())
}

async fn handle_datagram(datagram: Datagram, game: &Arc<Mutex<Game>>, id: u64) {
    let payload = datagram.payload();

    let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);

    // Logger::debug(format!(
    //     "Received input '({x:.2}, {y:.2})' from client {id}"
    // ));

    let mut game = game.lock().await;
    let _ = game.update_player_input(id, Vec2::new(x, y)).await;
}

async fn handle_leaderboard_update(
    update: LeaderboardUpdate,
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

    send_chat_message(request, connection).await
}

//

const FORBIDDEN_PLAYER_NAME_CHARACTERS: [char; 8] = ['#', '@', '$', '^', ':', '/', '\\', '*'];

fn validate_player_name(name: &str) -> bool {
    name.chars()
        .all(|c| !FORBIDDEN_PLAYER_NAME_CHARACTERS.contains(&c))
}

async fn send_chat_message(request: ChatRequest, connection: &Connection) -> Result<()> {
    let data = request.to_bytes();

    let mut stream = connection.open_uni().await?.await?;
    stream.write_all(&data).await?;
    stream.finish().await?;

    Ok(())
}

async fn spawn_hero(
    name: &str,
    connection: &Connection,
    game: &Arc<Mutex<Game>>,
    id: u64,
) -> Result<Vec<u8>> {
    let mut game = game.lock().await;
    let leaderboard_state = game.leaderboard_state.clone();

    game.spawn_hero(id, name, connection.clone()).await;

    let area_key = &game.get_player(id)?.area_key;
    let area = game.get_or_create_area(area_key)?;

    send_definition_stream(area, connection).await?;

    let mut response = Vec::new();

    response.extend_from_slice(&id.to_le_bytes());
    response.extend_from_slice(&leaderboard_state.to_bytes());

    Ok(response)
}

async fn send_definition_stream(area: Arc<Mutex<Area>>, connection: &Connection) -> Result<()> {
    let definition = area.lock().await.definition_packet();

    let mut definition_stream = connection.open_uni().await?.await?;
    definition_stream.write_all(&definition).await?;
    definition_stream.finish().await?;

    Ok(())
}

fn handle_unknown_header(header: &[u8], id: u64) {
    let header = match std::str::from_utf8(header) {
        Ok(header) => header,
        Err(_) => &format!("0x{:x?}", header),
    };

    Logger::warn(format!(
        "Received unknown packet from client @{id} (header: {header})"
    ));
}

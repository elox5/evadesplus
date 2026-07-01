use anyhow::Result;
use arc_swap::ArcSwap;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::{
    collections::HashMap,
    future::Future,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};
use tokio::sync::{broadcast, mpsc, Mutex};
use warp::{
    filters::ws::{self, WebSocket},
    Filter,
};

use crate::{
    logger::{LogCategory, Logger},
    networking::new::{
        client_id::ClientId,
        client_message::ClientMessage,
        server_message::{ServerMessage, ServerMessageTarget},
    },
};

pub trait ConnectionManager {
    fn serve(self) -> impl Future<Output = Result<()>> + Send + Sync;

    fn client_messages(&self) -> broadcast::Receiver<ClientMessage>;
    fn server_messages(&self) -> mpsc::Sender<ServerMessage>;
}

static NEXT_CLIENT_ID: AtomicU16 = AtomicU16::new(1);

pub struct WsConnectionManager {
    addr: SocketAddr,

    client_tx: broadcast::Sender<ClientMessage>,
    client_rx: broadcast::Receiver<ClientMessage>,

    server_tx: mpsc::Sender<ServerMessage>,

    connection_map: Arc<ArcSwap<WsConnectionMap>>,
}

impl WsConnectionManager {
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        let (client_tx, client_rx) = broadcast::channel(64);
        let (server_tx, server_rx) = mpsc::channel(64);

        let map = WsConnectionMap::default();
        let map_arc = Arc::new(ArcSwap::from_pointee(map));
        let map_arc_clone = map_arc.clone();

        tokio::task::spawn(async move {
            Self::process_server_messages(server_rx, map_arc_clone).await;
        });

        Self {
            addr: addr.into(),
            client_tx,
            client_rx,
            server_tx,
            connection_map: map_arc,
        }
    }

    async fn handle_connection(
        ws: WebSocket,
        client_tx: broadcast::Sender<ClientMessage>,
        connection_map: Arc<ArcSwap<WsConnectionMap>>,
    ) {
        let id: ClientId = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

        let (user_sink, mut user_stream) = ws.split();

        let sink: Arc<Mutex<SplitSink<WebSocket, ws::Message>>> = Arc::new(Mutex::new(user_sink));

        connection_map.rcu(|map| {
            let mut map = (**map).clone();
            map.insert(id, sink.clone());
            map
        });

        Logger::info(format!("WebSocket connection @{id} established"));

        while let Some(msg) = user_stream.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    Logger::error(format!("WebSocket error for user @{id}: {e}"));
                    break;
                }
            };

            if msg.is_close() {
                Logger::info(format!("WebSocket connection @{id} closed"));

                let msg = ClientMessage::new(id, "CLSE", Vec::new());
                let _ = client_tx.send(msg);

                connection_map.rcu(|map| {
                    let mut map = (**map).clone();
                    map.remove(&id);
                    map
                });

                break;
            }

            let bytes = msg.as_bytes();
            let (header, data) = bytes.split_at(4);

            let msg = ClientMessage::new(id, header, data.to_vec());

            let _ = client_tx.send(msg);
        }
    }

    async fn process_server_messages(
        mut srx: mpsc::Receiver<ServerMessage>,
        map: Arc<ArcSwap<WsConnectionMap>>,
    ) {
        while let Some(message) = srx.recv().await {
            let map = map.load();

            let targets = match message.target.clone() {
                ServerMessageTarget::All => map.get_all(),
                ServerMessageTarget::Single(id) => Vec::from([map.get(id).unwrap()]),
                ServerMessageTarget::Group(ids) => {
                    ids.iter().map(|id| map.get(*id).unwrap()).collect()
                }
            };

            let mut bytes: Vec<u8> = Vec::new();
            bytes.extend_from_slice(&message.header.bytes);
            bytes.extend_from_slice(&message.data);

            Logger::log(
                format!(
                    "Sending {} message to clients {:?}",
                    message.header.to_string(),
                    message.target
                ),
                LogCategory::Network,
            );

            let ws_message = ws::Message::binary(bytes);

            for sink in targets {
                let mut sink = sink.lock().await;

                let result = sink.send(ws_message.clone()).await;

                if let Err(e) = result {
                    Logger::error(format!("Failed to send message: {e}"));
                }
            }
        }
    }
}

impl ConnectionManager for WsConnectionManager {
    async fn serve(self) -> Result<()> {
        let client_tx = self.client_tx.clone();
        let with_client_tx = warp::any().map(move || client_tx.clone());

        let map = self.connection_map.clone();
        let with_map = warp::any().map(move || map.clone());

        let route = warp::path::end()
            .and(warp::ws())
            .and(with_client_tx)
            .and(with_map)
            .map(move |ws: warp::ws::Ws, message_tx, map| {
                ws.on_upgrade(move |socket| async move {
                    Self::handle_connection(socket, message_tx, map).await;
                })
            });

        warp::serve(route).run(self.addr).await;

        Ok(())
    }

    fn client_messages(&self) -> broadcast::Receiver<ClientMessage> {
        self.client_rx.resubscribe()
    }

    fn server_messages(&self) -> mpsc::Sender<ServerMessage> {
        self.server_tx.clone()
    }
}

type WsSink = SplitSink<WebSocket, ws::Message>;

#[derive(Default, Clone)]
struct WsConnectionMap {
    map: HashMap<ClientId, Arc<Mutex<WsSink>>>,
}

impl WsConnectionMap {
    fn get(&self, id: ClientId) -> Option<Arc<Mutex<WsSink>>> {
        self.map.get(&id).cloned()
    }

    fn get_all(&self) -> Vec<Arc<Mutex<WsSink>>> {
        self.map.values().cloned().collect()
    }

    fn insert(&mut self, id: ClientId, tx: Arc<Mutex<WsSink>>) {
        self.map.insert(id, tx);
    }

    fn remove(&mut self, id: &ClientId) {
        self.map.remove(id);
    }
}

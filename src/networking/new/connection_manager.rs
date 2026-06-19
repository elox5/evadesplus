use anyhow::Result;
use std::{future::Future, net::SocketAddr};
use warp::Filter;

use crate::logger::Logger;

pub trait ConnectionManager {
    fn serve(self) -> impl Future<Output = Result<()>> + Send + Sync;
}

pub struct WsConnectionManager {
    path: String,
    addr: SocketAddr,
}

impl WsConnectionManager {
    pub fn new(path: String, addr: impl Into<SocketAddr>) -> Self {
        Self {
            path,
            addr: addr.into(),
        }
    }
}

impl ConnectionManager for WsConnectionManager {
    async fn serve(self) -> Result<()> {
        let route = warp::path(self.path)
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                ws.on_upgrade(move |_socket| async move {
                    Logger::info(format!("Received message from WebSocket client"));
                })
            });

        warp::serve(route).run(self.addr).await;

        Ok(())
    }
}

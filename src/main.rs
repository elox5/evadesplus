use anyhow::Result;
use evadesplus::{
    cache::Cache,
    config::CONFIG,
    game::{
        game::{Game, GameOutputMessage},
        map_table::get_map_list,
    },
    logger::{LogCategory, Logger},
    networking::{
        chat::Chat,
        leaderboard::{Leaderboard, LeaderboardStore},
        new::{
            connection_manager::{ConnectionManager, WsConnectionManager},
            handlers::{
                client_chat_handler::ClientChatHandler, client_message_logger::ClientMessageLogger,
                close_handler::CloseHandler, handler::ClientMessageHandler,
                init_handler::InitHandler, render_handler::RenderHandler,
            },
            server_message::{ServerMessage, ServerMessageTarget},
            user_registry::create_user_registry,
        },
    },
};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::Mutex;
use warp::hyper::Uri;
use warp::Filter;
use wtransport::Identity;

#[tokio::main]
async fn main() -> Result<()> {
    let chat = Chat::new();
    let leaderboard = Leaderboard::new();

    let lb_store = LeaderboardStore::new();
    let lb_store = Arc::new(Mutex::new(lb_store));

    let user_registry = create_user_registry();

    let game = Game::new();

    let network_config = &CONFIG.network;

    if !std::path::Path::new(&network_config.client_path).is_dir() {
        panic!("Client code has not been compiled.");
    }

    let cache = Cache::new(get_map_list());
    let cache_hash = cache.get_hash();

    let connection_manager = WsConnectionManager::new(SocketAddr::new(
        IpAddr::V4(network_config.ip),
        network_config.ws_port,
    ));

    {
        let mut client_rx = connection_manager.client_messages().resubscribe();
        let client_message_logger =
            ClientMessageLogger::new(vec!["PING".to_owned(), "MOVE".to_owned()]);

        tokio::task::spawn(async move {
            while let Ok(message) = client_rx.recv().await {
                if client_message_logger.accept_header(&message.header) {
                    let _ = client_message_logger.handle(message);
                }
            }
        });
    }

    {
        let mut client_rx = connection_manager.client_messages().resubscribe();
        let chat_handler = ClientChatHandler::new(chat.tx.clone(), user_registry.clone());

        tokio::task::spawn(async move {
            while let Ok(message) = client_rx.recv().await {
                if chat_handler.accept_header(&message.header) {
                    let _ = chat_handler.handle(message);
                }
            }
        });
    }

    {
        let mut client_rx = connection_manager.client_messages().resubscribe();
        let server_tx = connection_manager.server_messages().clone();

        let init_handler = InitHandler::new(
            user_registry.clone(),
            server_tx,
            leaderboard.tx.clone(),
            lb_store.clone(),
            game.clone(),
            chat.tx.clone(),
        );

        tokio::task::spawn(async move {
            while let Ok(message) = client_rx.recv().await {
                if init_handler.accept_header(&message.header) {
                    let _ = init_handler.handle(message).await;
                }
            }
        });
    }

    {
        let mut client_rx = connection_manager.client_messages().resubscribe();
        let lb_tx = leaderboard.tx.clone();
        let chat_tx = chat.tx.clone();
        let close_handler = CloseHandler::new(user_registry.clone(), lb_tx, chat_tx);

        tokio::task::spawn(async move {
            while let Ok(message) = client_rx.recv().await {
                if close_handler.accept_header(&message.header) {
                    let _ = close_handler.handle(message);
                }
            }
        });
    }

    {
        let mut chat_rx = chat.rx.resubscribe();
        let server_tx = connection_manager.server_messages().clone();

        tokio::task::spawn(async move {
            while let Ok(message) = chat_rx.recv().await {
                Logger::log(
                    format!("{}: {}", message.sender_name, message.message),
                    LogCategory::Chat,
                );

                let bytes = message.to_bytes();

                let response = ServerMessage {
                    header: "CHAT".into(),
                    data: bytes,
                    target: ServerMessageTarget::All,
                };

                let _ = server_tx.send(response).await;
            }
        });
    }

    {
        let mut lb_rx = leaderboard.rx.resubscribe();
        let lb_store = lb_store.clone();
        let server_tx = connection_manager.server_messages().clone();

        tokio::spawn(async move {
            while let Ok(update) = lb_rx.recv().await {
                lb_store.lock().await.update(update.clone());

                let msg = ServerMessage {
                    header: update.header().as_str().into(),
                    data: update.to_bytes(),
                    target: ServerMessageTarget::All,
                };

                let _ = server_tx.send(msg).await;
            }
        });
    }

    {
        let mut game_rx = game.output_rx.resubscribe();
        let server_tx = connection_manager.server_messages().clone();
        let render_handler = RenderHandler {
            users: user_registry.clone(),
            server_tx,
        };

        tokio::spawn(async move {
            while let Ok(message) = game_rx.recv().await {
                match message {
                    GameOutputMessage::AreaRender(message) => {
                        let _ = render_handler.handle_render(message).await;
                    }
                    GameOutputMessage::AreaDefinition(message) => {
                        let _ = render_handler.handle_area_definition(message).await;
                    }
                }
            }
        });
    }

    let identity =
        Identity::load_pemfiles(&network_config.ssl_cert_path, &network_config.ssl_key_path)
            .await
            .unwrap_or_else(|err| {
                Logger::warn(format!("Failed to load SSL certificate: {err}"));
                Logger::warn("Generating self-signed certificate... (browsers might react oddly)");

                Identity::self_signed([&network_config.ip.to_string()]).unwrap()
            });

    let cert = identity.certificate_chain().as_slice()[0].clone();
    let cert = cert.to_pem();

    let key = identity.private_key().clone_key();
    let key = key.to_secret_pem();

    let root_route = warp::fs::dir(network_config.client_path.clone());
    let cache_route = warp::path("cache").and(warp::get()).then(move || {
        let cache = cache.clone();
        async move { warp::reply::json(&cache) }
    });
    let cache_hash_route = warp::path("cache_hash").and(warp::get()).then(move || {
        let hash = cache_hash.clone();
        async move { warp::reply::json(&hash) }
    });
    let wt_port_route = warp::path("wt_port").and(warp::get()).then(move || {
        let port = CONFIG.network.webtransport_port;
        async move { warp::reply::json(&port) }
    });

    let routes = root_route
        .or(cache_route)
        .or(cache_hash_route)
        .or(wt_port_route);

    let http_redirect_uri = Uri::from_str(&format!(
        "https://{}:{}",
        &network_config.ip, &network_config.client_port_https
    ))?;

    let http_route = warp::any().map(move || warp::redirect(http_redirect_uri.clone()));

    let https_addr = SocketAddr::new(
        IpAddr::V4(network_config.ip),
        network_config.client_port_https,
    );
    let http_addr = SocketAddr::new(
        IpAddr::V4(network_config.ip),
        network_config.client_port_http,
    );

    Logger::info(&format!("HTTP server litening on https://{https_addr}"));

    tokio::select! {
        _result = warp::serve(http_route).run(http_addr) => {
            Logger::info("HTTP server closed");
        }
        _result = warp::serve(routes).tls().cert(cert).key(key).run(https_addr) => {
            Logger::info("HTTPS server closed");
        }
        _result = connection_manager.serve() => {
            Logger::info("Connection manager closed");
        }
    }

    Ok(())
}

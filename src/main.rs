use anyhow::Result;
use evadesplus::{
    cache::Cache,
    config::CONFIG,
    game::{
        game::{GameCreator, GameOutputMessage},
        map_table::get_map_list,
    },
    logger::{LogCategory, Logger},
    networking::{
        chat::{Chat, ChatMessageType, ChatRequest},
        commands::{CommandRequest, handle_command},
        helpers::create_server_announcement,
        leaderboard::{Leaderboard, LeaderboardStore, LeaderboardUpdate},
        new::{
            connection_manager::{ConnectionManager, WsConnectionManager},
            handlers::{
                client_chat_handler::ClientChatHandler, client_message_logger::ClientMessageLogger,
                close_handler::CloseHandler, handler::ClientMessageHandler,
                init_handler::InitHandler, move_handler::MoveHandler,
                render_handler::RenderHandler,
            },
            server_message::{ServerMessage, ServerMessageTarget},
            user_registry::{UserId, create_user_registry},
        },
    },
};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::Mutex;
use warp::Filter;
use warp::hyper::Uri;
use wtransport::Identity;

#[tokio::main]
async fn main() -> Result<()> {
    let chat = Chat::new();
    let leaderboard = Leaderboard::new();

    let lb_store = LeaderboardStore::new();
    let lb_store = Arc::new(Mutex::new(lb_store));

    let user_registry = create_user_registry();

    let game = GameCreator::new().create_game();

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
        let close_handler = CloseHandler::new(user_registry.clone(), lb_tx, chat_tx, game.clone());

        tokio::task::spawn(async move {
            while let Ok(message) = client_rx.recv().await {
                if close_handler.accept_header(&message.header) {
                    let _ = close_handler.handle(message).await;
                }
            }
        });
    }

    {
        let mut client_rx = connection_manager.client_messages().resubscribe();
        let move_handler = MoveHandler::new(user_registry.clone(), game.clone());

        tokio::spawn(async move {
            while let Ok(message) = client_rx.recv().await {
                if move_handler.accept_header(&message.header) {
                    let _ = move_handler.handle(message).await;
                }
            }
        });
    }

    {
        let mut chat_rx = chat.rx.resubscribe();
        let server_tx = connection_manager.server_messages().clone();
        let game = game.clone();
        let users = user_registry.clone();

        tokio::task::spawn(async move {
            while let Ok(message) = chat_rx.recv().await {
                Logger::log(
                    format!("{}: {}", message.sender_name, message.message),
                    LogCategory::Chat,
                );

                let bytes = message.to_bytes();

                if message.message.starts_with('/') {
                    let text = &message.message[1..];
                    let splits = text.split(" ").collect::<Vec<&str>>();
                    let command = splits[0];
                    let args = splits[1..].iter().map(|s| s.to_string()).collect();

                    let req = CommandRequest {
                        args,
                        game: game.clone(),
                        users: users.clone(),
                        user_id: message.sender_id.clone(),
                    };

                    let response = handle_command(command, req).await;

                    let message = match response {
                        Ok(response) => response,
                        Err(err) => Some(ChatRequest::new(
                            format!(
                                "A server error has occurred. Please report it to the developers: *{err:?}*"
                            ),
                            String::new(),
                            UserId(u64::MAX),
                            ChatMessageType::ServerError,
                            Some(vec![message.sender_id]),
                        )),
                    };

                    if let Some(message) = message {
                        let response = ServerMessage {
                            header: "CHAT".into(),
                            data: message.to_bytes(),
                            target: ServerMessageTarget::All,
                        };

                        let _ = server_tx.send(response).await;
                    }
                } else {
                    let response = ServerMessage {
                        header: "CHAT".into(),
                        data: bytes,
                        target: ServerMessageTarget::All,
                    };

                    let _ = server_tx.send(response).await;
                }
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
        let lb_tx = leaderboard.tx.clone();
        let chat_tx = chat.tx.clone();
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
                    GameOutputMessage::PlayerTransfer(message) => {
                        let users = user_registry.clone();

                        if let Some(user_id) = users.player_to_user_id(&message.player_id) {
                            Logger::debug(format!(
                                "Updating player id from '{}' to '{}'",
                                message.player_id, message.new_id
                            ));

                            let _ = lb_tx.send(LeaderboardUpdate::transfer(
                                user_id.clone(),
                                message.area_info.clone(),
                            ));

                            users.update_player_id(user_id.clone(), message.new_id.clone());

                            if let Some(user) = users.get(&user_id) {
                                let new_area = &message.new_id.area;

                                if message.area_info.victory && !user.victories.contains(new_area) {
                                    users.push_victory(&user_id, new_area);

                                    if let Some(timer) = message.timer {
                                        let minutes = timer.0 / 60.0;
                                        let seconds = (timer.0.floor() as u32) % 60;

                                        let announcement = create_server_announcement(format!(
                                            "{} just completed {} in {:02.0}:{:02.0}!",
                                            user.name, message.route_name, minutes, seconds
                                        ));

                                        let _ = chat_tx.send(announcement);
                                    } else {
                                        Logger::error(
                                            "Expected Timer component on hero when transferring to victory area",
                                        );
                                    }
                                }
                            }
                        }
                    }
                    GameOutputMessage::PlayerReset(player_id) => {
                        let users = user_registry.clone();

                        if let Some(user_id) = users.player_to_user_id(&player_id) {
                            users.clear_victories(&user_id);
                        }
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

use super::chat::{ChatMessageType, ChatRequest};
use crate::cache::CommandCache;
use crate::game::game::Game;
use crate::game::game::TransferRequest;
use crate::game::game::TransferTarget;
use crate::game::map_table::map_exists;
use anyhow::anyhow;
use anyhow::Result;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, LazyLock},
};
use tokio::sync::Mutex;

static COMMANDS: LazyLock<Vec<Command>> = LazyLock::new(|| {
    vec![
        Command::new(
            "help",
            Some(vec!["h", "?"]),
            "Displays the list of available commands, or info about a specific command.",
            Some("<command?>"),
            None,
        ),
        Command::new("clear", None, "Clears all chat messages.", None, None),
        Command::new(
            "reset",
            Some(vec!["r"]),
            "Resets the player.",
            None,
            Some(Box::new(reset)),
        ),
        Command::new(
            "disconnect",
            Some(vec!["dc", "ff", "quit"]),
            "Disconnects from the server.",
            None,
            None,
        ),
        Command::new(
            "whisper",
            Some(vec!["w", "pm", "msg", "message"]),
            "Sends a private message to another player.",
            Some("<player> <message>"),
            Some(Box::new(whisper)),
        ),
        Command::new(
            "reply",
            Some(vec!["re"]),
            "Sends a private message to the last player who whispered to you.",
            Some("<message>"),
            None,
        ),
        Command::new(
            "togglereply",
            Some(vec!["autoreply", "togglere", "tr", "togglew", "tw"]),
            "Tries to automatically sends a private reply when sending a chat message.",
            None,
            None,
        ),
        Command::new(
            "warp",
            Some(vec!["tp"]),
            "Warps you to the start of the provided map",
            Some("<map>"),
            Some(Box::new(warp)),
        ),
        Command::new(
            "filter",
            None,
            "Filters the chat to only show messages from the given map",
            Some("<map? | \"off\">"),
            None,
        ),
    ]
});

struct Command {
    name: String,
    aliases: Option<Vec<String>>,
    description: String,
    usage: Option<String>,
    handler: Option<Box<dyn AsyncFn>>,
}

impl Command {
    pub fn new(
        name: &str,
        aliases: Option<Vec<&str>>,
        description: &str,
        usage: Option<&str>,
        handler: Option<Box<dyn AsyncFn>>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            aliases: aliases.map(|a| a.iter().map(|s| (*s).to_owned()).collect()),
            description: description.to_owned(),
            usage: usage.map(|s| s.to_owned()),
            handler,
        }
    }

    pub fn matches(&self, command_name: &str) -> bool {
        if self.name == command_name {
            return true;
        }

        if let Some(aliases) = &self.aliases {
            for alias in aliases {
                if alias == command_name {
                    return true;
                }
            }
        }

        false
    }

    pub async fn execute(&self, req: CommandRequest) -> Result<Option<ChatRequest>> {
        if let Some(handler) = &self.handler {
            handler.call(req).await
        } else {
            Err(anyhow!(
                "The /{} command should have been handled on the client.",
                self.name
            ))
        }
    }
}

pub async fn handle_command(
    command_name: &str,
    req: CommandRequest,
) -> Result<Option<ChatRequest>> {
    for command in COMMANDS.iter() {
        if command.matches(&command_name.to_lowercase()) {
            return command.execute(req).await;
        }
    }

    Err(anyhow!(
        "Unknown command: /{}. This should've been handled on the client, but it has somehow reached the server.",
        &command_name.to_lowercase()
    ))
}

pub struct CommandRequest {
    pub args: Vec<String>,
    pub game: Arc<Mutex<Game>>,
    pub player_id: u64,
}

async fn reset(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let mut game = req.game.lock().await;
    game.reset_hero(req.player_id).await?;

    Ok(None)
}

async fn whisper(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let sender_id = req.player_id;

    let recipient_text = match req.args.get(0) {
        Some(text) => text,
        None => {
            return response("You must specify a target player".to_owned(), sender_id);
        }
    };

    let game = req.game.lock().await;

    let maybe_recipient = if recipient_text.starts_with('@') {
        let recipient_id = recipient_text[1..].parse::<u64>();

        if let Ok(id) = recipient_id {
            game.get_player(id)
        } else {
            return response(
                format!("{recipient_text} is not a valid player ID"),
                sender_id,
            );
        }
    } else {
        game.get_player_by_name(recipient_text)
    };

    let recipient = match maybe_recipient {
        Ok(recipient) => recipient,
        Err(err) => {
            return response(err.to_string(), sender_id);
        }
    };

    let message = req.args[1..].join(" ");

    if message.is_empty() {
        return response("Whisper message cannot be empty.".to_owned(), sender_id);
    }

    if recipient.id == sender_id {
        return response("You cannot whisper to yourself.".to_owned(), sender_id);
    }

    let player = game.get_player(sender_id)?;

    Ok(Some(ChatRequest::new(
        message,
        format!("{} -> {}", player.name, recipient.name),
        sender_id,
        ChatMessageType::Whisper,
        Some(vec![sender_id, recipient.id]),
    )))
}

async fn warp(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let map_id = match req.args.get(0) {
        Some(id) => id,
        None => return response("You must specify a target map".to_owned(), req.player_id),
    };

    if !map_exists(map_id) {
        return response(format!("Map '{}' does not exist", map_id), req.player_id);
    }

    let transfer_request = TransferRequest {
        player_id: req.player_id,
        target: TransferTarget::MapStart(map_id.to_owned()),
        target_pos: None,
    };

    let mut game = req.game.lock().await;

    game.transfer_hero(transfer_request).await?;

    Ok(None)
}

//

fn response(message: String, recipient_id: u64) -> Result<Option<ChatRequest>> {
    Ok(Some(ChatRequest::new(
        message,
        String::new(),
        u64::MAX,
        ChatMessageType::CommandResponse,
        Some(vec![recipient_id]),
    )))
}

pub fn get_command_cache() -> Vec<CommandCache> {
    COMMANDS
        .iter()
        .map(|c| {
            CommandCache::new(
                c.name.clone(),
                c.description.clone(),
                c.usage.clone(),
                c.aliases.clone(),
            )
        })
        .collect()
}

// unholy magic

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;

pub trait AsyncFn: Send + Sync {
    fn call(&self, args: CommandRequest) -> BoxFuture<'static, Result<Option<ChatRequest>>>;
}

impl<T, F> AsyncFn for T
where
    T: Fn(CommandRequest) -> F + Send + Sync,
    F: Future<Output = Result<Option<ChatRequest>>> + 'static + Send + Sync,
{
    fn call(&self, args: CommandRequest) -> BoxFuture<'static, Result<Option<ChatRequest>>> {
        Box::pin(self(args))
    }
}

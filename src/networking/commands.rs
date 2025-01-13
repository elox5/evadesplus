use super::chat::{ChatMessageType, ChatRequest};
use crate::game::game::Game;
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
            Some(vec!["h"]),
            "Displays this help message.",
            None,
            Box::new(help),
        ),
        Command::new(
            "reset",
            Some(vec!["r"]),
            "Resets the player.",
            None,
            Box::new(reset),
        ),
        Command::new(
            "whisper",
            Some(vec!["w", "pm", "msg", "message"]),
            "Sends a private message to another player.",
            Some("/whisper <player> <message>"),
            Box::new(whisper),
        ),
    ]
});

pub fn get_command_list_binary() -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(b"CMDL"); // 4 bytes
    bytes.push(COMMANDS.len() as u8); // 1 byte

    for command in COMMANDS.iter() {
        bytes.push(command.name.len() as u8); // 1 byte
        bytes.extend_from_slice(command.name.as_bytes()); // name.len() bytes

        bytes.extend_from_slice(&(command.description.len() as u16).to_le_bytes()); // 2 bytes
        bytes.extend_from_slice(command.description.as_bytes()); // help_description.len() bytes

        if let Some(usage) = &command.usage {
            bytes.extend_from_slice(&(usage.len() as u16).to_le_bytes()); // 2 bytes
            bytes.extend_from_slice(usage.as_bytes()); // usage.len() bytes
        } else {
            bytes.extend_from_slice(&(0u16).to_le_bytes()); // 2 bytes
        }

        if let Some(aliases) = &command.aliases {
            bytes.push(aliases.len() as u8); // 1 byte
            for alias in aliases {
                bytes.push(alias.len() as u8); // 1 byte
                bytes.extend_from_slice(alias.as_bytes()); // alias.len() bytes
            }
        } else {
            bytes.push(0); // 1 byte
        }
    }

    bytes
}

struct Command {
    name: String,
    aliases: Option<Vec<String>>,
    description: String,
    usage: Option<String>,
    function: Box<dyn AsyncFn>,
}

impl Command {
    pub fn new(
        name: &str,
        aliases: Option<Vec<&str>>,
        description: &str,
        usage: Option<&str>,
        function: Box<dyn AsyncFn>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            aliases: aliases.map(|a| a.iter().map(|s| (*s).to_owned()).collect()),
            description: description.to_owned(),
            usage: usage.map(|s| s.to_owned()),
            function,
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
        self.function.call(req).await
    }
}

pub async fn handle_command(
    command_name: &str,
    req: CommandRequest,
) -> Result<Option<ChatRequest>> {
    for command in COMMANDS.iter() {
        if command.matches(command_name) {
            return command.execute(req).await;
        }
    }

    Err(anyhow!(
        "Unknown command: /{command_name}. This should've been handled on the client, but it has somehow reached the server."
    ))
}

pub struct CommandRequest {
    pub args: Vec<String>,
    pub game: Arc<Mutex<Game>>,
    pub player_id: u64,
}

async fn help(_req: CommandRequest) -> Result<Option<ChatRequest>> {
    Err(anyhow!(
        "The /help command should have been handled on the client."
    ))
}

async fn reset(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let mut game = req.game.lock().await;
    game.reset_hero(req.player_id).await?;

    Ok(None)
}

async fn whisper(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let sender_id = req.player_id;

    let recipient_name = match req.args.get(0) {
        Some(name) => name,
        None => {
            return response("You must specify a target player".to_owned(), sender_id);
        }
    };

    let game = req.game.lock().await;

    let recipient = match game.get_player_by_name(&recipient_name) {
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
        format!("{} -> {}", player.name, recipient_name),
        sender_id,
        ChatMessageType::Whisper,
        Some(vec![sender_id, recipient.id]),
    )))
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

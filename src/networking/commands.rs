use super::chat::{ChatMessageType, ChatRequest};
use crate::game::game::{Game, Player};
use anyhow::Result;
use arc_swap::ArcSwap;
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
            Box::new(help),
        ),
        Command::new(
            "reset",
            Some(vec!["r"]),
            "Resets the player.",
            Box::new(reset),
        ),
        Command::new(
            "whisper",
            Some(vec!["w", "pm"]),
            "Sends a private message to another player.",
            Box::new(whisper),
        ),
    ]
});

struct Command {
    name: String,
    aliases: Option<Vec<String>>,
    help_description: String,
    function: Box<dyn AsyncFn>,
}

impl Command {
    pub fn new(
        name: &str,
        aliases: Option<Vec<&str>>,
        help_description: &str,
        function: Box<dyn AsyncFn>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            aliases: aliases.map(|a| a.iter().map(|s| (*s).to_owned()).collect()),
            help_description: help_description.to_owned(),
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

    response(
        format!(
            "Unknown command: */{command_name}*. For a list of available commands, use */help*."
        ),
        req.player.load().id,
    )
}

pub struct CommandRequest {
    args: Vec<String>,
    game: Arc<Mutex<Game>>,
    player: Arc<ArcSwap<Player>>,
}

impl CommandRequest {
    pub fn new(args: Vec<String>, game: Arc<Mutex<Game>>, player: Arc<ArcSwap<Player>>) -> Self {
        Self { args, game, player }
    }
}

async fn help(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let mut messages = Vec::new();

    for command in COMMANDS.iter() {
        let mut msg = format!("*/{}* - {}", command.name, command.help_description);

        if let Some(aliases) = &command.aliases {
            let aliases = aliases
                .iter()
                .map(|a| format!("/{}", a))
                .collect::<Vec<_>>();

            msg.push_str("\nAliases: ");
            msg.push_str(&aliases.join(", "));
        }

        messages.push(msg);
    }

    let help_message = messages.join("\n\n");

    Ok(Some(ChatRequest::new(
        help_message,
        String::new(),
        ChatMessageType::CommandResponse,
        Some(vec![req.player.load().id]),
    )))
}

async fn reset(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let mut game = req.game.lock().await;
    game.reset_hero(&req.player).await?;

    Ok(None)
}

async fn whisper(req: CommandRequest) -> Result<Option<ChatRequest>> {
    let recipient_name = req.args[0].clone();

    let game = req.game.lock().await;

    let recipient = if let Some(recipient) = game.get_player_by_name(&recipient_name) {
        recipient
    } else {
        return response(
            format!("Player '{}' not found.", recipient_name),
            req.player.load().id,
        );
    };

    let player = req.player.load();

    let message = req.args[1..].join(" ");

    if message.is_empty() {
        return response("Whisper message cannot be empty.".to_owned(), player.id);
    }

    if recipient.id == player.id {
        return response("You cannot whisper to yourself.".to_owned(), player.id);
    }

    Ok(Some(ChatRequest::new(
        message,
        format!("{} -> {}", player.name, recipient_name),
        ChatMessageType::Whisper,
        Some(vec![player.id, recipient.id]),
    )))
}

//

fn response(message: String, recipient_id: u64) -> Result<Option<ChatRequest>> {
    Ok(Some(ChatRequest::new(
        message,
        String::new(),
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

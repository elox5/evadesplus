use super::chat::{ChatMessageType, ChatRequest};
use crate::game::game::{Game, Player};
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
            "repeat",
            None,
            "Repeats the input to the command.",
            Box::new(repeat),
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

    pub async fn execute(&self, req: CommandRequest) -> Option<ChatRequest> {
        self.function.call(req).await
    }
}

pub async fn handle_command(command_name: &str, req: CommandRequest) -> Option<ChatRequest> {
    for command in COMMANDS.iter() {
        if command.matches(command_name) {
            return command.execute(req).await;
        }
    }

    Some(ChatRequest::new(
        format!(
            "Unknown command: <b>/{command_name}</b>. For a list of available commands, use <b>/help</b>."
        ),
        String::new(),
        ChatMessageType::CommandResponse,
        Some(vec![req.player.load().id]),
    ))
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

async fn help(req: CommandRequest) -> Option<ChatRequest> {
    let mut messages = Vec::new();

    for command in COMMANDS.iter() {
        let mut msg = format!("<b>/{}</b> - {}", command.name, command.help_description);

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

    Some(ChatRequest::new(
        help_message,
        String::new(),
        ChatMessageType::CommandResponse,
        Some(vec![req.player.load().id]),
    ))
}

async fn reset(req: CommandRequest) -> Option<ChatRequest> {
    let mut game = req.game.lock().await;
    let result = game.reset_hero(&req.player).await;

    result.err().map(|err| {
        ChatRequest::new(
            format!(
                "A server error has occurred. Please report it to the developers: <b>{err:?}</b>"
            ),
            String::new(),
            ChatMessageType::ServerError,
            Some(vec![req.player.load().id]),
        )
    })
}

async fn repeat(req: CommandRequest) -> Option<ChatRequest> {
    Some(ChatRequest::new(
        req.args.join(" "),
        String::new(),
        ChatMessageType::CommandResponse,
        Some(vec![req.player.load().id]),
    ))
}

async fn whisper(req: CommandRequest) -> Option<ChatRequest> {
    let recipient_name = req.args[0].clone();

    let game = req.game.lock().await;
    let recipient = game.get_player_by_name(&recipient_name);

    let player = req.player.load();

    if let Some(recipient) = recipient {
        let message = req.args[1..].join(" ");

        if message.is_empty() {
            return Some(ChatRequest::new(
                "Whisper message cannot be empty.".to_owned(),
                String::new(),
                ChatMessageType::CommandResponse,
                Some(vec![player.id]),
            ));
        }

        return Some(ChatRequest::new(
            message,
            format!("{} -> {}", player.name, recipient_name),
            ChatMessageType::Whisper,
            Some(vec![recipient.id, player.id]),
        ));
    }

    return Some(ChatRequest::new(
        format!("Player '{recipient_name}' not found."),
        String::new(),
        ChatMessageType::CommandResponse,
        Some(vec![player.id]),
    ));
}

// unholy magic

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;

pub trait AsyncFn: Send + Sync {
    fn call(&self, args: CommandRequest) -> BoxFuture<'static, Option<ChatRequest>>;
}

impl<T, F> AsyncFn for T
where
    T: Fn(CommandRequest) -> F + Send + Sync,
    F: Future<Output = Option<ChatRequest>> + 'static + Send + Sync,
{
    fn call(&self, args: CommandRequest) -> BoxFuture<'static, Option<ChatRequest>> {
        Box::pin(self(args))
    }
}

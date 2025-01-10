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
        Command::new("help", "Displays this help message.", Box::new(help)),
        Command::new("reset", "Resets the player.", Box::new(reset)),
        Command::new(
            "repeat",
            "Repeats the input to the command.",
            Box::new(repeat),
        ),
    ]
});

struct Command {
    name: String,
    help_description: String,
    function: Box<dyn AsyncFn>,
}

impl Command {
    pub fn new(name: &str, help_description: &str, function: Box<dyn AsyncFn>) -> Self {
        Self {
            name: name.to_owned(),
            help_description: help_description.to_owned(),
            function,
        }
    }

    pub async fn execute(&self, req: CommandRequest) -> Option<ChatRequest> {
        self.function.call(req).await
    }
}

pub async fn handle_command(command_name: &str, req: CommandRequest) -> Option<ChatRequest> {
    for command in COMMANDS.iter() {
        if command.name == command_name {
            return command.execute(req).await;
        }
    }

    Some(ChatRequest::new(
        format!(
            "Unknown command: <b>/{command_name}</b>. For a list of available commands, use <b>/help</b>."
        ),
        String::new(),
        ChatMessageType::CommandResponse,
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

async fn help(_req: CommandRequest) -> Option<ChatRequest> {
    let mut help_message = String::new();
    for command in COMMANDS.iter() {
        help_message.push_str(&format!(
            "<b>{}</b> - {}\n",
            command.name, command.help_description
        ));
    }

    Some(ChatRequest::new(
        help_message,
        String::new(),
        ChatMessageType::CommandResponse,
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
        )
    })
}

async fn repeat(req: CommandRequest) -> Option<ChatRequest> {
    Some(ChatRequest::new(
        req.args.join(" "),
        String::new(),
        ChatMessageType::CommandResponse,
    ))
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

use crate::networking::{
    chat::{ChatMessageType, ChatRequest},
    new::user_registry::UserId,
};

const FORBIDDEN_PLAYER_NAME_CHARACTERS: [char; 8] = ['#', '@', '$', '^', ':', '/', '\\', '*'];

pub fn validate_player_name(name: &str) -> bool {
    name.chars()
        .all(|c| !FORBIDDEN_PLAYER_NAME_CHARACTERS.contains(&c))
}

pub fn create_server_announcement(message: String) -> ChatRequest {
    ChatRequest::new(
        message,
        String::new(),
        UserId(u64::MAX),
        ChatMessageType::ServerAnnouncement,
        None,
    )
}

const FORBIDDEN_PLAYER_NAME_CHARACTERS: [char; 8] = ['#', '@', '$', '^', ':', '/', '\\', '*'];

pub fn validate_player_name(name: &str) -> bool {
    name.chars()
        .all(|c| !FORBIDDEN_PLAYER_NAME_CHARACTERS.contains(&c))
}

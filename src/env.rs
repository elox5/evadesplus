pub fn get_env_var(name: &str) -> String {
    dotenvy::var(name).expect(&format!(".env {name} must be set"))
}

pub fn get_env_or_default(name: &str, default: &str) -> String {
    dotenvy::var(name).unwrap_or_else(|_| default.to_owned())
}

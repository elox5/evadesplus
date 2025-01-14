pub fn get_env_var(name: &str) -> String {
    dotenvy::var(name).expect(&format!(".env {name} must be set"))
}

fn main() {
    // build client typescript
    std::process::Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("client")
        .spawn()
        .expect("Failed to run tsc");
}

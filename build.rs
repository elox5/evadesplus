fn main() {
    // build client typescript
    let tsc = std::process::Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("client")
        .spawn()
        .ok();

    if tsc.is_none() {
        println!("Failed to build client typescript");
    }
}

use std::env;
use std::fs;

fn main() {
    let target = env::var("TARGET_ENV").unwrap();
    println!("Target: {}", target);
    if target != "macos" {
        let mut content = fs::read_to_string("Cargo.toml").expect("Unable to read Cargo.toml");

        // Remove the crate-type if present
        if content.contains("crate-type = [\"cdylib\"]") {
            content = content.replace("crate-type = [\"cdylib\"]", "");
        }

        fs::write("Cargo.toml", content).expect("Unable to write Cargo.toml");
    }
}

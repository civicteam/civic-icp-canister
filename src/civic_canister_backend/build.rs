use std::env;
use std::fs;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("Target: {}", target);
    // When we build for Ubuntu on CI, we need to remove the crate-type otherwise it fails with linker issues
    // We need the crate-type for the wasm build
    if target == "ubuntu" {
        let mut content = fs::read_to_string("Cargo.toml").expect("Unable to read Cargo.toml");

        // Remove the crate-type if present
        if content.contains("crate-type = [\"cdylib\", \"lib\"]") {
            content = content.replace("crate-type = [\"cdylib\", \"lib\"]", "");
        }

        fs::write("Cargo.toml", content).expect("Unable to write Cargo.toml");
    }
}

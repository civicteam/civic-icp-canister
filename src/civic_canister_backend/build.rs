use std::env;
use std::fs;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("linux-gnu") {
        let mut content = fs::read_to_string("Cargo.toml").expect("Unable to read Cargo.toml");

        // Remove the crate-type if present
        if content.contains("crate-type = [\"cdylib\"]") {
            content = content.replace("crate-type = [\"cdylib\"]", "");
        }

        fs::write("Cargo.toml", content).expect("Unable to write Cargo.toml");
    }
}

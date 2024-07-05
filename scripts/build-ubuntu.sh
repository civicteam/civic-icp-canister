#!/bin/bash

CARGO_TOML="./src/civic_canister_backend/Cargo.toml"

# Check if the Cargo.toml file exists
if [[ ! -f $CARGO_TOML ]]; then
    echo "Cargo.toml file not found!"
    exit 1
fi

# Read the Cargo.toml file
cargo_toml_content=$(cat $CARGO_TOML)

# Remove the crate-type line if it exists
updated_content=$(echo "$cargo_toml_content" | sed '/crate-type = \["cdylib", "lib"\]/d')

# Write the updated content back to Cargo.toml
echo "$updated_content" > $CARGO_TOML

echo "crate-type line removed from Cargo.toml"

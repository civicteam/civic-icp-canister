#!/bin/bash

# Define the output location
TARGET_DIR=./target
OUTPUT_DIR=./src/civic_canister_backend/

# Build the wasm file
cargo build --target wasm32-unknown-unknown --release --target-dir $TARGET_DIR

# Create the output directory if it doesn't exist
mkdir -p $OUTPUT_DIR

# Copy the wasm file to the output directory
cp $TARGET_DIR/wasm32-unknown-unknown/release/civic_canister_backend.wasm $OUTPUT_DIR/

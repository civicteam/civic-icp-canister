#!/bin/bash

# Define the output location
TARGET_DIR=./target
OUTPUT_DIR=./src/civic_canister_backend/
TARGET_ENV=macos

# Export the TARGET environment variable
export TARGET_ENV=$TARGET_ENV

# Print the TARGET environment variable to verify it is set
echo "TARGET is set to: $$TARGET_ENV"

# Build the wasm file with env command to ensure the environment variable is picked up
env TARGET=$TARGET cargo build --target wasm32-unknown-unknown --release --target-dir $TARGET_DIR

# Check if the build was successful
if [ $? -ne 0 ]; then
  echo "Cargo build failed"
  exit 1
fi

# Create the output directory if it doesn't exist
mkdir -p $OUTPUT_DIR

# Copy the wasm file to the output directory
cp $TARGET_DIR/wasm32-unknown-unknown/release/civic_canister_backend.wasm $OUTPUT_DIR/

# Verify the wasm file was copied successfully
if [ -f $OUTPUT_DIR/civic_canister_backend.wasm ]; then
  echo "WASM file successfully copied to $OUTPUT_DIR"
else
  echo "Failed to copy WASM file"
  exit 1
fi

#!/usr/bin/env bash
set -e

# Build the plugin using the wasm32-unknown-unknown target
cargo build --target wasm32-unknown-unknown --release

# Ensure plugins directory exists in ~/.seabubble
mkdir -p ~/.seabubble/plugins/

# Copy the compiled .wasm file
cp target/wasm32-unknown-unknown/release/demo_scanner.wasm ~/.seabubble/plugins/demo-scanner.wasm

echo "Successfully built demo-scanner.wasm and copied to ~/.seabubble/plugins/"

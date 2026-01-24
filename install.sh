#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "Building release..."
cargo build --release

mkdir -p ~/.local/bin

echo "Installing to ~/.local/bin/dig-json..."
cp target/release/dig-json ~/.local/bin/dig-json

echo "Done."

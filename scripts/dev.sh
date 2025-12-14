#!/bin/bash
# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "cargo-watch could not be found, installing..."
    cargo install cargo-watch
fi

cargo watch -x run

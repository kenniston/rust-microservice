#!/bin/sh

# Script Name: execute-build.sh
# Author: Kenniston Arraes Bonfim
# Date: February 4, 2026
# Version: 1.0
# Description: This script builds the project based on the Cargo.toml file.
#
set -e

validation() {
    echo "ℹ️ Validating Cargo.toml file..."
    cargo check --quiet
    echo "✅ Cargo.toml file validated successfully!"
}

build() {
    echo "ℹ️ Running cargo build..."
    export RUSTFLAGS="-C link-arg=-s"
    export OPENSSL_STATIC=1
    cargo build -p server --quiet --release --target x86_64-unknown-linux-musl

    echo "ℹ️ Copying binary to .binaries folder..."
    mkdir -p .binaries
    cp ./target/x86_64-unknown-linux-musl/release/server .binaries/server

    if [ -n "$BINARY_COMPRESSION" ] && [ "$BINARY_COMPRESSION" = "true" ]; then
        echo "ℹ️ Compressing binary with UPX..."
        upx --ultra-brute -v .binaries/server && upx -t .binaries/server
    fi

    echo "✅ Build completed successfully!"
}


validation
build

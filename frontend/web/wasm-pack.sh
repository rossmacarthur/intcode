#!/usr/bin/env bash

set -e

WORKSPACE_DIR=$(cargo metadata --format-version=1 | jq -r .workspace_root)

CARGO_VERSION=$(cargo metadata --format-version=1 | jq -r '.packages[] | select (.name == "wasm-bindgen") | .version')
CLI_VERSION=$(wasm-bindgen --version | awk '{print $2}')
if [ "$CARGO_VERSION" != "$CLI_VERSION" ]; then
    echo "differing wasm-bindgen versions" >&2
    exit 1
fi

rm -rf "$WORKSPACE_DIR/frontend/web/dist" "$WORKSPACE_DIR/frontend/web/wasm"

RUSTFLAGS="-C opt-level=z" \
    cargo build --release --package intcode-wasm --target wasm32-unknown-unknown

wasm-bindgen \
    --target=web \
    --out-dir="$WORKSPACE_DIR/frontend/web/wasm" \
    "$WORKSPACE_DIR/target/wasm32-unknown-unknown/release/intcode_wasm.wasm"

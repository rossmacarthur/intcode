#!/usr/bin/env bash

set -e

ROOT_DIR=$(cargo metadata --format-version=1 | jq -r .workspace_root)

rm -rf "$ROOT_DIR/web/dist" "$ROOT_DIR/web/wasm"

RUSTFLAGS="-C opt-level=z" \
    cargo build --release --package intcode-wasm --target wasm32-unknown-unknown

wasm-bindgen \
    --target=web \
    --out-dir="$ROOT_DIR/web/wasm" \
    "$ROOT_DIR/target/wasm32-unknown-unknown/release/intcode_wasm.wasm"

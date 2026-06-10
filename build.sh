#!/bin/bash

set -e;

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

# Useful for debugging:
# export CARGO_PROFILE_RELEASE_DEBUG=2
# export WIT_BINDGEN_DEBUG=1

mkdir -p "${SCRIPT_DIR}/lib"

wasm-tools component wit --wasm "${SCRIPT_DIR}/wit" -o "${SCRIPT_DIR}/lib/package.wasm"

cargo build -p adapter --target wasm32-unknown-unknown --release
cp "${SCRIPT_DIR}/target/wasm32-unknown-unknown/release/adapter.wasm" "${SCRIPT_DIR}/lib/adapter.wasm"

cargo build -p factory --target wasm32-unknown-unknown --release
wasm-tools component new "${SCRIPT_DIR}/target/wasm32-unknown-unknown/release/factory.wasm" -o "${SCRIPT_DIR}/lib/factory.wasm"

cargo build --release

cargo test

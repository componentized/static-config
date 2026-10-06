#!/bin/bash

set -e;

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

mkdir -p "${SCRIPT_DIR}/lib"

wkg oci pull ghcr.io/componentized/constants/config:0.1.0-dev -o "${SCRIPT_DIR}/lib/config.wasm"

cargo build -p factory --target wasm32-unknown-unknown --profile component
wasm-tools component new "${SCRIPT_DIR}/target/wasm32-unknown-unknown/component/factory.wasm" -o "${SCRIPT_DIR}/lib/factory.wasm"

cargo build --release

cargo test

#!/bin/bash

set -e;

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

mkdir -p "${SCRIPT_DIR}/lib"

# the config component matching the version of the componentized-constants crate
constants_version=$(cargo pkgid componentized-constants | sed 's/.*@//')
wkg oci pull "ghcr.io/componentized/constants/config:${constants_version}" -o "${SCRIPT_DIR}/lib/config.wasm"

cargo build -p factory --target wasm32-unknown-unknown --profile component
wasm-tools component new "${SCRIPT_DIR}/target/wasm32-unknown-unknown/component/factory.wasm" -o "${SCRIPT_DIR}/lib/factory.wasm"

cargo build --release

cargo test

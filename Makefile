SHELL := /bin/bash

export RUST_BACKTRACE ?= 1
export WASMTIME_BACKTRACE_DETAILS ?= 1
export WKG_CONFIG_FILE := $(abspath .config/wasm-pkg/config.toml)

# `make` without a goal runs all, otherwise the first target of an included file would run
.DEFAULT_GOAL := all

# tools first, the others use the variables it defines for each tool when their rules are read
include make/tools.mk

# the static-config cli is built from this repository rather than pinned in tools/Cargo.toml,
# installed with the pinned tools, the components are built with it
STATIC_CONFIG := $(TOOLS_BIN)/static-config

# the constants-config component the library embeds, pulled by the dep-constants-config component. Its bytes are inlined
# into the cli and the factory component, so it's committed with the library crate, a published
# crate can only include its own files, and the crates build with cargo alone.
CONFIG_ADAPTER_WASM := crates/componentized-static-config/constants-config.wasm

include make/wit.mk
include make/components.mk
include make/publish.mk

# the toolchain pinned in rust-toolchain.toml, named for `cargo install`, otherwise rustup warns that
# the toolchain file overrides the default
RUST_TOOLCHAIN := $(shell sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)


.PHONY: all
all: tools wit components test

.PHONY: build
build: config
	cargo build --release

.PHONY: install
install: config
	cargo +$(RUST_TOOLCHAIN) install --path . --locked

.PHONY: clean
clean: clean-components clean-wit
	@:

.PHONY: clean-all
clean-all: clean-components clean-tools clean-wit
	cargo clean

.PHONY: test
test: components
	cargo test --workspace


.PHONY: config ## Update the config component the library embeds from dep-constants-config
config: $(CONFIG_ADAPTER_WASM)

# the config component must match the version of the componentized-constants crate in Cargo.lock
$(CONFIG_ADAPTER_WASM): ${COMPONENTS_DIR}/dep-constants-config/dep-constants-config.wasm Cargo.lock
	@# the tag of the first image, with or without a digest, e.g. `...config:0.1.1@sha256:...`
	@component_version=$$(sed -n '1{s/@.*//;s/.*://;p;}' components/dep-constants-config/dep-constants-config.wkg) ; \
	crate_version=$$(cargo pkgid componentized-constants | sed 's/.*@//') ; \
	if [ "$${component_version}" != "$${crate_version}" ] ; then \
		echo "the config component $${component_version} in components/dep-constants-config/dep-constants-config.wkg does not match componentized-constants $${crate_version} in Cargo.lock" >&2 ; \
		exit 1 ; \
	fi
	cp ${COMPONENTS_DIR}/dep-constants-config/dep-constants-config.wasm $(CONFIG_ADAPTER_WASM)


tools: $(STATIC_CONFIG)

$(STATIC_CONFIG): $(CONFIG_ADAPTER_WASM) Cargo.toml Cargo.lock rust-toolchain.toml $(shell find src crates/componentized-static-config -type f)
	@# forced, make only installs when the sources change, and the binary may belong to another package
	cargo +$(RUST_TOOLCHAIN) install --force --locked --root $(TOOLS_DIR) --path .
	@# cargo leaves the binary alone when it is already up to date
	@touch $@

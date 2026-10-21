# This repository builds with its own build, its components and cli exist to smoke test it, see
# README.md

# the cli, released with an archive for each platform, and published to crates.io
CLI_BIN := static-config
PUBLISH_CRATES := componentized-static-config componentized-static-config-cli

# the cli installed as a tool built from the workspace, like a cli the components are built with, e.g.
# the constants cli in componentized/constants, run as `$(HELLO)`
LOCAL_TOOLS := componentized-static-config-cli

# the constants-config component the library embeds, pulled by the dep-constants-config component. Its bytes are inlined
# into the cli and the factory component, so it's committed with the library crate, a published
# crate can only include its own files, and the crates build with cargo alone.
CONFIG_ADAPTER_WASM := crates/componentized-static-config/constants-config.wasm

include build/build.mk

generate: $(CONFIG_ADAPTER_WASM)

.PHONY: install ## install constants cli to CARGO_INSTALL_ROOT
install: $(CONFIG_ADAPTER_WASM)
	cargo +$(RUST_TOOLCHAIN) install --path . --locked --force

.PHONY: build
build: $(CONFIG_ADAPTER_WASM)
	cargo build --release

$(call tool,$(componentized-static-config-cli)): $(CONFIG_ADAPTER_WASM)

# the config component must match the version of the componentized-constants crate in Cargo.lock
$(CONFIG_ADAPTER_WASM): ${COMPONENTS_DIR}/dep-constants-config/dep-constants-config.wasm Cargo.lock
	@# the tag of the first image, with or without a digest, e.g. `0.1.1` for `...config:0.1.1@sha256:...`
	@component_version=$$(sed -n '1{s/@.*//;s/.*://;p;}' components/dep-constants-config/dep-constants-config.wkg) ; \
	crate_version=$$(cargo pkgid componentized-constants | sed 's/.*@//') ; \
	if [ "$${component_version}" != "$${crate_version}" ] ; then \
		echo "the config component $${component_version} in components/dep-constants-config/dep-constants-config.wkg does not match componentized-constants $${crate_version} in Cargo.lock" >&2 ; \
		exit 1 ; \
	fi
	cp ${COMPONENTS_DIR}/dep-constants-config/dep-constants-config.wasm $(CONFIG_ADAPTER_WASM)

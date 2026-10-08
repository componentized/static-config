# Static config components <!-- omit in toc -->

Create custom `wasi:config` components with static values.

- [Build](#build)
- [Community](#community)
  - [Code of Conduct](#code-of-conduct)
  - [Communication](#communication)
  - [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [License](#license)


## Build

A [dev container](https://containers.dev) is available that contains the necessary tools and configuration out of the box.

Prereqs:
- a rust toolchain
- [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall), optional, to download prebuilt tools instead of building them

The other tools the build uses, e.g. [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools) and [`wkg`](https://github.com/bytecodealliance/wasm-pkg-tools), are installed into `target/tools` by the make targets that need them, at the versions pinned in [`tools/Cargo.toml`](./tools/Cargo.toml). With [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) they're downloaded rather than built.

```sh
make components
```

The build creates each component in [`components`](./components) into `target/components`, e.g. the factory at `target/components/factory/factory.wasm`, along with `target/components/interface.wasm`, the `componentized:static-config` WIT package. Each component is also built with debug info, e.g. `target/components/factory/factory.debug.wasm`.

To run the tests, which exercise the library and the components:

```sh
make test
```

The library embeds the [`config`](https://github.com/componentized/constants/tree/main/components/config) component from Constants, which serves the values of every component it creates, so the CLI and the factory component include it too. It's pulled by the [`dep-constants-config`](./components/dep-constants-config) component, and committed at `crates/componentized-static-config/constants-config.wasm` so the crates build with `cargo` alone. Its version must match the `componentized-constants` crate in `Cargo.lock`. After bumping that crate, update the image in [`dep-constants-config.wkg`](./components/dep-constants-config/dep-constants-config.wkg) to match, and copy it into the library with:

```sh
make config
```

The WIT dependencies in each `wit/deps` directory are fetched rather than committed, pinned by the `wkg.lock` files. The make targets fetch them as needed. To fetch or update them directly, e.g. before building the Rust components with `cargo`, whose bindings are generated from the WIT:

```sh
make wit
```

## Community

### Code of Conduct

The Componentized project follow the [Contributor Covenant Code of Conduct](./CODE_OF_CONDUCT.md). In short, be kind and treat others with respect.

### Communication

General discussion and questions about the project can occur in the project's [GitHub discussions](https://github.com/orgs/componentized/discussions).

### Contributing

The Componentized project team welcomes contributions from the community. A contributor license agreement (CLA) is not required. You own full rights to your contribution and agree to license the work to the community under the Apache License v2.0, via a [Developer Certificate of Origin (DCO)](https://developercertificate.org). For more detailed information, refer to [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgements

This project was conceived in discussion between [Mark Fisher](https://github.com/markfisher) and [Scott Andrews](https://github.com/scothis).

Components are created with [Constants](https://github.com/componentized/constants), composing a component that returns the configured values with its [`config`](https://github.com/componentized/constants/tree/main/components/config) component, which exports them as `wasi:config/store`.

## License

Apache License v2.0: see [LICENSE](./LICENSE) for details.

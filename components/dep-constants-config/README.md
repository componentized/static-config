# `dep-constants-config`

The [`config`](https://github.com/componentized/constants/tree/main/components/config) component from Constants, embedded by the library as `crates/componentized-static-config/constants-config.wasm`. It serves the values of every component the library creates, so the cli and the factory component include it too.

Its version must match the version of the `componentized-constants` crate the library depends on, checked by `make config`.

## Interfaces

Imports:

- `componentized:constants/config-values@0.1.1`: the key value pairs to serve

Exports:

- `wasi:config/store@0.2.0-rc.1`: gets config values from the imported key value pairs

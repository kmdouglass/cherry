# Cherry

[![docs.rs](https://img.shields.io/docsrs/cherry-rs)](https://docs.rs/cherry-rs/latest/cherry_rs/)
[![Crates.io Version](https://img.shields.io/crates/v/cherry-rs)](https://crates.io/crates/cherry-rs)

Rust library for designing sequential optical systems.

## Features

### serde

Enables [serde](https://serde.rs) serialization and deserialization for all public types, plus `serde_json` support for the `SurfaceSpec::Custom` variant and `SurfaceRegistry`. Pure Rust library users who build models programmatically via `SequentialModel::from_surfaces` can omit this feature to avoid the serde dependency.

```toml
cherry-rs = { version = "*", features = [ "serde" ] }
```

### gui

Enables the egui-based desktop and WASM GUI. Implies `serde`.

```toml
cherry-rs = { version = "*", features = [ "gui" ] }
```

### ri-info

Includes material refractive index data from [refractiveindex.info](https://refractiveindex.info). Implies `serde`.

```toml
cherry-rs = { version = "*", features = [ "ri-info" ] }
```

#### Testing

Test data for the feature must be obtained from [refractiveindex.info-adapters](https://github.com/kmdouglass/refractiveindex.info-adapters/releases) in bitcode format. The resulting file should be placed in the `data/rii.db` directory.

## Development

### Requirements

1. Rust compiler - See [rust-toolchain.toml](../../../rust-toolchain.toml) for the version

### Testing

Run all tests, including all features. See [the Features section](#features) for obtaining test data.

```console
cargo test --all-features
```

Run tests for the core library only. No test data is necessary.

```console
cargo test
```

### Linting and Formatting

```console
cargo clippy --all-features
cargo fmt
```

### Benchmarks

These should be run under the same "conditions" for meaningful comparisons, i.e. the same hardware, CPU load, etc.

```console
cargo bench
```

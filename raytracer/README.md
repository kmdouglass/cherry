# Cherry

[![docs.rs](https://img.shields.io/docsrs/cherry-rs)](https://docs.rs/cherry-rs/latest/cherry_rs/)
[![Crates.io Version](https://img.shields.io/crates/v/cherry-rs)](https://crates.io/crates/cherry-rs)

Tools for designing sequential optical systems.

## Features

### ri-info

This feature includes material refractive index data from [refractiveindex.info](https://refractiveindex.info).

Install this feature by adding the following to your Cargo.toml: `cherry-rs = { version = "*", features = [ "ri-info" ]}`

#### Testing

Test data for the feature must be generated using [refractiveindex.info-adapters](https://github.com/kmdouglass/refractiveindex.info-adapters) in bitcode format. The resulting file should be placed in the `data/rii.db` directory.

## Development

### Requirements

1. Rust compiler - See [rust-toolchain.toml](rust-toolchain.toml) for the version
2. [wasmtime](https://wasmtime.dev/) - For testing the Rust crate in the `wasm32-wasip1` environment
3. [wasm-bindgen-cli](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/usage.html#appendix-using-wasm-bindgen-test-without-wasm-pack) - The version must match the installed version of `wasm-bindgen`

### Testing

Run all tests, including all features. See [the Features section](#features) for obtaining test data.

```console
cargo test --all-features
```

Run tests for the core library only. No test data is necessary.

```console
cargo test
```

#### Testing WASM Targets

Make sure that you have the requirements listed above installed.

Run all tests for cherry-rs including features for the wasm32-wasip1 target:

```console
cargo test -p cherry-rs --target wasm32-wasip1 --all-features
```

Run all tests for cherry-js for the wasm32-unknown-unknown target:

```console
cargo test -p cherry-js --target wasm32-unknown-unknown
```

### Benchmarks

These should be run under the same "conditions" for meaningful comparisons, i.e. the same hardware, CPU load, etc.

```console
cargo bench
```

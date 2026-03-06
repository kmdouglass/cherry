# Cherry Ray Tracer

Optical system design in the browser

*This is alpha software. Emphasis is currently placed on feature development, not on fixing bugs or on improving code quality.*

## Quickstart

1. Go to https://kmdouglass.github.io/cherry/
2. Start designing!

## Prerequisites

- Rust compiler (see [rust-toolchain.toml](rust-toolchain.toml) for the version)
- [trunk](https://trunk-rs.github.io/trunk/) (for the WASM frontend)
- [binaryen](https://github.com/WebAssembly/binaryen) (optional — trunk will use `wasm-opt` for WASM optimisation if it is on your PATH)

## Build

```console
cd www/egui
trunk build --release
```

## Development

Launch the development server:

```console
cd www/egui
trunk serve
```

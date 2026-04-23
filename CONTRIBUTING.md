# Contributing

Thank you so much for considering a contribution to Cherry. I really appreciate the time and effort that you are willing to invest in making this tool better.

## Table of Contents

1. [Before Opening a Pull Request](#before-opening-a-pull-request)
2. [Development](#development)

## Before Opening a Pull Request

Your ideas and suggestions are always welcome! Before working on a new feature, I would first like to know whether it fits within the scope of Cherry. I ask that contributors [create a feature request](https://github.com/kmdouglass/cherry/discussions/new?category=ideas) so that we can discuss your idea.

## Development

This workflow will set up the environment to work on the Cherry app. See [crates/cherry-rs/README.md](crates/cherry-rs/README.md) for information specific to the cherry-rs crate.

### Prerequisites

- Rust compiler (see [rust-toolchain.toml](rust-toolchain.toml) for the version)
- [trunk](https://trunk-rs.github.io/trunk/) (for the WASM frontend)
- [binaryen](https://github.com/WebAssembly/binaryen) (optional — trunk will use `wasm-opt` for WASM optimization if it is on your PATH)
- [just](https://github.com/casey/just) (command runner for running common development commands)
- [rii.db](https://github.com/kmdouglass/refractiveindex.info-adapters/releases) (the materials database)

Place the `rii.db` materials database in

1. `crates/cherry-rs/data` for desktop development and testing, and
2. in `www/egui/assets` for web app development.

### Set Up the Environment

First, you need to have the Rust tooling installed such as `cargo` and `rustup`. You can find installers and more information at [https://rustup.rs/](https://rustup.rs/) as well as the [rust-lang](https://rust-lang.org/) website.

Next, install a nightly version of the Rust compiler and the `wasm32-unknown-unknown` target. Be sure to check the [rust-toolchain.toml](rust-toolchain.toml) file for the current version and replace `nightly-2025-11-25` in the command below with the up-to-date version.

```console
rustup toolchain install nightly-2025-11-15
rustup target add wasm32-unknown-unknown
```

> [!NOTE]
> Nightly Rust is required because this project uses WebAssembly threads (which requires the unstable `atomics` feature) and unstable features of `rustfmt`.

> [!NOTE]
> In the unlikely event that you will need to change the compiler flags for the WASM app, they can be found at [/www/egui/.cargo/config.toml](/www/egui/.cargo/config.toml).

Once Rust is set up, install just, trunk, and binaryen (optional) using the instructions on their respective sites linked in the [Prerequisites section](#prerequisites). Trunk is the build system for the WASM app, and binaryen will optimize the WASM binary as long as it is on your PATH.

### Verify the Environment

If the command below succeeds, then you are ready to proceed to building and serving the WASM app.

```console
just check
```

### Serve Locally

After completing the prerequisites, you can build a development version of the WASM app and serve it locally:

```console
just serve
```

To run a development version of the desktop app, run:

```console
just gui
```

### Common Development Commands

The most useful command is `just ci`, which will

- format the code,
- run Clippy, and
- run all tests.

`just --list` will list all the available just commands. Finally, `just bench-all` will run all benchmark tests, which is useful to see whether your changes have introduced a performance regression.

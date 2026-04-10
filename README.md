# Cherry Ray Tracer

Optical system design in the browser

*This is alpha software. Emphasis is currently placed on feature development, not on fixing bugs or on improving code quality.*

## Quickstart

1. Go to https://kmdouglass.github.io/cherry/
2. Start designing!

## Development

See [crates/cherry-rs/README.md](crates/cherry-rs/README.md) for information about the cherry-rs library.

### Prerequisites

- Rust compiler (see [rust-toolchain.toml](rust-toolchain.toml) for the version)
- [trunk](https://trunk-rs.github.io/trunk/) (for the WASM frontend)
- [binaryen](https://github.com/WebAssembly/binaryen) (optional — trunk will use `wasm-opt` for WASM optimization if it is on your PATH)
- [rii.db](https://github.com/kmdouglass/refractiveindex.info-adapters/releases) (the materials database)

Place the `rii.db` materials database in

1. `crates/cherry-rs/data` for desktop development and testing, and
2. in `www/egui/assets` for web app development.

### Build

```console
cd www/egui
trunk build --release
```

### Serve Locally

Launch the development server:

```console
cd www/egui
trunk serve
```

## License

Copyright (c) 2024-2026, ECOLE POLYTECHNIQUE FEDERALE DE LAUSANNE, Switzerland, Laboratory of Experimental Biophysics (LEB)

The cherry-rs library is licensed under the [GNU Lesser General Public License v3.0 or later](LICENSE.txt) (LGPL-3.0-or-later). The compiled `cherry` GUI binary is licensed under the GNU General Public License v3.0 or later (GPL-3.0-or-later).

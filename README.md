# Cherry Ray Tracer

Optical system design in the browser

*This is alpha software. Emphasis is currently placed on feature development, not on fixing bugs or on improving code quality.*

## Quickstart

1. Go to https://kmdouglass.github.io/cherry/
2. Start designing!

## Prerequisites

- Rust compiler (see [raytracer/rust-toolchain.toml](raytracer/rust-toolchain.toml) for the version)
- Node 20 (for the frontend)
- [Git LFS](https://git-lfs.com/) (to fetch materials data)

## Build

```console
cd www/js
npm run build:prod
```

## Development

Build the development package:

```console
cd www/js
npm build:dev
```

Launch the development server

```
npm run start
```

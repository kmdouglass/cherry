# Cherry

Interactive sequential optical ray tracer that runs in the browser. Users design and analyze optical systems (lenses, mirrors, apertures) in real-time. Alpha-stage software; prioritize features over polish.

Focus on:

- Interactivity: See changes, graphical and numeric, immediately in the browser frontend
  - Speed matters; long calculations to be performed off the main thread
  - Keep the user up-to-date about what is going on
- Sensible defaults; don't repeat idiosyncracies of commercial optical design software, i.e. Zemax

## Architecture

Three-layer stack: Rust core -> WASM bridge -> Web frontend.

- `raytracer/cherry-rs/` — Core Rust library: ray tracing, paraxial analysis, surface geometry, material database. This is the most important part of the codebase.
- `raytracer/cherry-js/` — WASM bindings via `wasm-bindgen`. Compiles cherry-rs to WebAssembly callable from JS.
- `www/js/` — React frontend with Web Worker for background computation. **This layer is being removed in favor of pure WASM in the browser.** Do not invest in new JS features or refactors unless explicitly asked.

## Build and test

Requires Rust nightly (`nightly-2025-02-06`, pinned in `raytracer/rust-toolchain.toml`), `wasm-pack`, `wasm-bindgen-cli`, and Node.js 22.

### Rust

```bash
# Run cherry-rs tests (preferred target for core lib)
cd raytracer && cargo test -p cherry-rs

# Run cherry-js WASM binding tests
cd raytracer && cargo test -p cherry-js --target wasm32-unknown-unknown

# Run benchmarks
cd raytracer && cargo bench

# Format
cd raytracer/cherry-rs && cargo fmt
```

### JavaScript

```bash
cd www/js
npm ci              # install dependencies
npm run test        # vitest
npm run build:prod  # production build (compiles WASM + bundles JS)
npm run start       # dev server
```

## Rust conventions

- Edition 2024, workspace resolver 3.
- Use `anyhow` for errors, `approx` for float comparisons in tests.
- Integration tests live in `raytracer/cherry-rs/tests/`. Unit tests use inline `#[cfg(test)]` modules.
- Formatting: `wrap_comments = true`, unstable rustfmt features enabled.
- Keep it simple; use fancy Rust constructs only when needed.
- Use few dependencies

## Key domain concepts

- **Sequential model**: Surfaces are traced in order along the optical axis. Each surface has a geometry (conic, sphere) and orientation (3D rotation).
- **Specs** (`specs/`): User inputs — `SurfaceSpec`, `GapSpec`, `FieldSpec`, `ApertureSpec`.
- **Views** (`views/`): Computed outputs — `ParaxialView` (first-order properties), `RayTrace3DView` (full trace), `CutawayView` (cross-section rendering).
- **Materials**: Refractive index data, optionally sourced from refractiveindex.info via the `ri_info` feature flag.

## Reference Docs
- `.claude/design/gui.md`: GUI design and implementation
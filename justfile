set working-directory := "crates"

alias b := bench-all
alias c := ci
alias f := fmt
alias g := gui
alias l := lint
alias s := serve
alias t := test-all

[doc("Run a specific benchmark test. Example: 'just bench f_theta_scan_lens'")]
bench bench_name:
  cargo bench --bench {{bench_name}}

[doc("Run all benchmark tests")]
bench-all:
  cargo bench

[doc("Build the WASM release package")]
[working-directory: "../www/egui"]
build-wasm:
  trunk build --release

check:
  cargo check

[doc("Run all CI-level checks")]
ci: fmt lint test-all

[doc("Launches the desktop GUI")]
gui:
  cargo run -p cherry-rs --bin cherry --features gui,ri-info

fmt:
  cargo fmt

lint:
  cargo clippy --all-features --all-targets

list-benches:
  cargo bench -- --list

list-tests:
  cargo test --test '*' -- --list

[doc("Build and serve the web app on a local web server")]
[working-directory: "../www/egui"]
serve:
  trunk serve

test-all:
  cargo test --all-features --all-targets

[doc("Run an integration test with tracing. Filter example: '[{ray_id=0}]=trace'")]
[arg("filter", long, short="f")]
[arg("test_name", long, short="n")]
trace test_mod filter="" test_name="":
  RUST_LOG="{{filter}}" cargo test --test {{test_mod}} {{test_name}} -- --nocapture

set working-directory := "crates"

alias f := fmt
alias g := gui
alias l := lint
alias s := serve
alias t := test

[doc("Run a benchmark test. Example: 'just bench f_theta_scan_lens'")]
bench bench_name:
  cargo bench --bench {{bench_name}}

[doc("Launches the desktop GUI")]
gui:
  cargo run -p cherry-rs --bin cherry --features gui

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

test:
  cargo test --all-features

[doc("Run an integration test with tracing. Filter example: '[{ray_id=0}]=trace'")]
[arg("filter", long, short="f")]
[arg("test_name", long, short="n")]
trace test_mod filter="" test_name="":
  RUST_LOG="{{filter}}" cargo test --test {{test_mod}} {{test_name}} -- --nocapture

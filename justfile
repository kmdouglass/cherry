[doc("Run a benchmark test. Example: 'just bench f_theta_scan_lens'")]
[working-directory: "crates"]
bench bench_name:
  cargo bench --bench {{bench_name}}

[working-directory: "crates"]
list-benches:
  cargo bench -- --list

[working-directory: "crates"]
list-tests:
  cargo test --test '*' -- --list

[doc("Run an integration test with tracing. Filter example: '[{ray_id=0}]=trace'")]
[arg("filter", long, short="f")]
[arg("test_name", long, short="n")]
[working-directory: "crates"]
trace test_mod filter="" test_name="":
  RUST_LOG="{{filter}}" cargo test --test {{test_mod}} {{test_name}} -- --nocapture

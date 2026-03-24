[doc("Run an integration test with tracing. Filter example: '[{ray_id=0}]=trace'")]
[arg("filter", long, short="f")]
[arg("test_name", long, short="n")]
[working-directory: "crates"]
trace test_mod filter="" test_name="":
  RUST_LOG="{{filter}}" cargo test --test {{test_mod}} {{test_name}} -- --nocapture

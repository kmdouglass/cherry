{
  description = "Optical ray tracing in the browser";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        inputs = [ rust pkgs.wasm-pack pkgs.curl];
      in
      {
        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "cherry";
          version = "1.0.0";

          src = self;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = inputs;

          buildPhase = ''
            export HOME=$PWD
            mkdir -p $out
            wasm-pack build --target bundler --out-dir $out
          '';
          dontCargoInstall = true;
        };

        devShell = pkgs.mkShell { packages = inputs; };
      }
    );
}

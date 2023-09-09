{
  description = "Optical ray tracing in the browser";

  inputs = {
    clj2nix = {
      url = "github:hlolli/clj2nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-compat.follows = "flake-compat";
      };
    };
    flake-compat = {
      url = github:edolstra/flake-compat;
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, clj2nix, flake-compat, flake-utils, nixpkgs, rust-overlay }:
    let
      version = "${nixpkgs.lib.substring 0 8 self.lastModifiedDate}.${self.shortRev or "dirty"}";
    in

    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust = pkgs.rust-bin.fromRustupToolchainFile ./raytracer/rust-toolchain.toml;
        inputs = [ rust pkgs.wasm-pack pkgs.curl ];

        clj-deps = import ./www/cljs/deps.nix { inherit (pkgs) fetchMavenArtifact fetchgit lib; };
        classpath = clj-deps.makeClasspaths { };

      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            clj2nix.defaultPackage.${system}
            clojure
            ghp-import
            yarn

            rust
            wasm-pack
            curl
          ];
        };

        packages = {
          raytracer = pkgs.rustPlatform.buildRustPackage {
            pname = "cherry";
            version = "1.0.0";

            src = ./raytracer;

            cargoLock = {
              lockFile = ./raytracer/Cargo.lock;
            };

            nativeBuildInputs = inputs;

            buildPhase = ''
              export HOME=$PWD
              mkdir -p $out
              wasm-pack build --target bundler --out-dir $out
            '';

            dontInstall = true;
            dontCargoInstall = true;
          };

          site = pkgs.stdenv.mkDerivation {
            name = "cherry-web-${version}";

            buildInputs = [ pkgs.clojure pkgs.yarn ];

            nodeModules = pkgs.mkYarnModules rec {
              pname = "cherry-web";
              name = "cherry-web-node-modules-${version}";
              inherit version;
              packageJSON = ./www/cljs/package.json;
              yarnLock = ./www/cljs/yarn.lock;
            };

            src = ./www/cljs;

            configurePhase = ''
              cp -r $nodeModules/node_modules .
              chmod +w node_modules

              ln -s ${self.packages.${system}.raytracer.out} node_modules/cherry
              ln -s ${./www/cljs/src/rendering} node_modules/rendering
            '';

            buildPhase = ''
              export HOME=$PWD
              clojure -Scp ${classpath}:src/main -M:shadow-cljs release app
              yarn --non-interactive --offline run build --mode=production
            '';

            installPhase = ''
              mkdir $out
              cp -r public/* $out
              cp -r dist/* $out
            '';
          };

          defaultPackage = self.packages.${system}.site;

        };
      });
}

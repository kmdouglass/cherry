# Cherry raytracer

## Prerequisite

Install the [Nix package manager](https://nixos.org/download.html).

## Build

From the repository's root directory:

```
nix build .#site
```

This will build the contents of the [website](http://browser.science).

## Development

First, build the raytracer's WASM module:

```
(cd raytracer && nix develop --command wasm-pack build)
```

Then, to develop the ClojureScript frontend interactively, install the dependencies and start the development server:

```
nix develop --command yarn workspace cherry-web install
nix develop --command yarn workspace cherry-web start
```

Finally, in a separate terminal, build the ClojureScript application:

```
(cd www/cljs && nix develop --command clj -M:shadow-cljs watch app)
```

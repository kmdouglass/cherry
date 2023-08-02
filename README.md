# cherry-web
Cherry Web Frontend

# Development

Interactive shell
```
nix develop --command yarn install
nix develop --command clj -M:shadow-cljs watch app
```

Update dependencies

```
nix develop --command clj2nix deps.edn deps.nix
```

# cherry-web
Cherry Web Frontend

# Development

Interactive development
```
nix develop --command yarn install
nix develop --command yarn start
nix develop --command clj -M:shadow-cljs watch app
```

Update dependencies

```
nix develop --command clj2nix deps.edn deps.nix
```

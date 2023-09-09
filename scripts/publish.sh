#! /usr/bin/env bash

set -eu

SITE=$(nix build .#site --no-link --print-out-paths)
nix run nixpkgs#ghp-import -- --message "Automatic update from https://github.com/wagdav/cherry-web" "$SITE"
git push --force origin gh-pages:gh-pages

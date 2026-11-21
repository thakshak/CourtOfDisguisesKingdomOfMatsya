#!/usr/bin/env bash
set -euo pipefail

cargo build --lib --target wasm32-unknown-unknown --release --offline

rm -rf docs
mkdir -p docs/assets
cp web/index.html web/styles.css web/app.js docs/
cp target/wasm32-unknown-unknown/release/battle_of_matsya.wasm docs/battle_of_matsya.wasm
cp -R assets/* docs/assets/
touch docs/.nojekyll

echo "GitHub Pages static site built in ./docs"

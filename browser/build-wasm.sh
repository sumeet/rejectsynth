#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
if [[ "$(wasm-bindgen --version)" != 'wasm-bindgen 0.2.100' ]]; then
  echo 'This build needs wasm-bindgen-cli 0.2.100 (matching Cargo.toml).' >&2
  echo 'Install with: cargo install wasm-bindgen-cli --version 0.2.100 --locked' >&2
  exit 1
fi
cargo build --manifest-path ../rejectsynth/Cargo.toml --locked --target wasm32-unknown-unknown --lib --release
wasm-bindgen --target web --out-dir generated ../rejectsynth/target/wasm32-unknown-unknown/release/rejectsynth.wasm

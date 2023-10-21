#!/bin/bash
set -ex

mkdir -p build/wasm

pushd rejectsynth
cargo build --target=wasm32-unknown-unknown --lib --release
wasm-bindgen --target=nodejs target/wasm32-unknown-unknown/release/rejectsynth.wasm --out-dir=../build/wasm --no-typescript
popd



#!/bin/bash
set -ex

mkdir -p build/wasm

pushd rejectsynth
cargo build --target=wasm32-unknown-unknown --lib
wasm-bindgen --target=nodejs target/wasm32-unknown-unknown/debug/rejectsynth.wasm --out-dir=../build/wasm --no-typescript
popd



#!/bin/bash
set -e
cd ..
cd "`dirname $0`"
cd ft
cargo build --all --target wasm32-unknown-unknown --release
cd ..
cp ft/target/wasm32-unknown-unknown/release/*.wasm ./res/

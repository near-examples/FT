@echo off
title FT build
cd ft
cargo build --target wasm32-unknown-unknown --release
copy target/wasm32-unknown-unknown/release/*.wasm ./res/
pause
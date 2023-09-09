#!/usr/bin/env bash
set -e

cargo build --workspace --release
cd links-wasm
wasm-pack build --target web --release
cd ..
cp links-wasm/pkg/links_wasm_bg.wasm html/
cp links-wasm/pkg/links_wasm.js html/


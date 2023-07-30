#!/usr/bin/env bash
#
set -e

wasm-pack build --target web
cp pkg/links_wasm_bg.wasm ../html/
cp pkg/links_wasm.js ../html/

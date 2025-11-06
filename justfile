run:
  RUST_LOG=info cargo run

compile-release:
  cargo build --workspace --release
  cd links-wasm && wasm-pack build --target web --release
  cp links-wasm/pkg/links_wasm_bg.wasm html/
  cp links-wasm/pkg/links_wasm.js html/


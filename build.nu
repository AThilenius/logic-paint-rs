#!/usr/bin/env nu

export def main [] {
  $env.RUSTFLAGS = '-C target-feature=+atomics,+bulk-memory,+mutable-globals'
  # cargo build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort
  cargo build --target wasm32-unknown-unknown -Z build-std=std,panic_abort
  # wasm-bindgen target/wasm32-unknown-unknown/release/logic_paint_rs.wasm --out-dir pkg --target web --debug --keep-debug
  wasm-bindgen target/wasm32-unknown-unknown/debug/logic_paint_rs.wasm --out-dir pkg --target web --debug --keep-debug

  # Unsure why wasm-bindgen doesn't creat this like wasm-pack does.
  '{
  "name": "logic-paint-rs",
  "collaborators": [
    "Alec Thilenius <alec@thilenius.com>"
  ],
  "description": "A logic simulation library written in Rust, inspired by KOHCTPYKTOP.",
  "version": "0.1.0",
  "license": "MIT/Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://gitlab.com/athilenius/logic-paint-rs"
  },
  "files": [
    "logic_paint_rs_bg.wasm",
    "logic_paint_rs.js",
    "logic_paint_rs.d.ts"
  ],
  "module": "logic_paint_rs.js",
  "types": "logic_paint_rs.d.ts",
  "sideEffects": [
    "./snippets/*"
  ]
}' | save --force pkg/package.json
}

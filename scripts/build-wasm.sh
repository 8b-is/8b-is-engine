#!/usr/bin/env bash
# build-wasm.sh — the third surface, materialized.
#
# Builds the ternary crate for wasm32-unknown-unknown with the simd128
# lane enabled (the official toolchain — the engine's wasm compiler) and
# drops the binary into client/assets/ternary.wasm, where the
# dream-dashboard page and `node client/dream.js --check` consume it.
#
# Usage: ./scripts/build-wasm.sh   (from the repo root)

set -euo pipefail
cd "$(dirname "$0")/.."

RUST_OFFICIAL="$HOME/.rust-official/bin"
RUSTC="${RUSTC:-$RUST_OFFICIAL/rustc}"
CARGO="${CARGO:-$RUST_OFFICIAL/cargo}"

if [ ! -x "$RUSTC" ] || [ ! -x "$CARGO" ]; then
  echo "build-wasm: the official toolchain ($RUST_OFFICIAL) is required" >&2
  exit 1
fi

mkdir -p client/assets
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-wasm}" \
  RUSTC="$RUSTC" CARGO="$CARGO" RUSTFLAGS="-C target-feature=+simd128" \
  "$CARGO" build -p ternary --target wasm32-unknown-unknown --release

cp "target-wasm/wasm32-unknown-unknown/release/ternary.wasm" client/assets/ternary.wasm
echo "⟦ third surface ⟧ client/assets/ternary.wasm ($(wc -c < client/assets/ternary.wasm) bytes, simd128)"

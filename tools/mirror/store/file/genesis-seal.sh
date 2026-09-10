#!/usr/bin/env bash
# genesis-seal.sh — compute the constellation's genesis seal: the public
# version-hash anchor an installer can verify against, without touching
# the repo. The seal records the engine's version, the canonical artifact
# hashes (the base model, the wasm surface), and the three-surface
# golden. Refreshed on every release: scripts/genesis-seal.sh > seal.json
# && gh gist edit <ID> seal.json
set -euo pipefail
cd "$(dirname "$0")/.."

version="$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -1)"
golden="$(grep -o 'aff6dc2[a-f0-9]*' crates/ternary/src/golden.rs | head -1)"
tern_sha="$(shasum -a 256 assets/ternary/sanctuary-1.58.tern | cut -d' ' -f1)"
wasm_sha="$(shasum -a 256 client/assets/ternary.wasm | cut -d' ' -f1)"

cat <<JSON
{
  "kind": "genesis-seal",
  "engine": "8b-is-engine",
  "version": "${version}",
  "sealed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "golden": "${golden}",
  "artifacts": {
    "sanctuary-1.58.tern": "${tern_sha}",
    "ternary.wasm": "${wasm_sha}"
  }
}
JSON

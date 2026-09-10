#!/usr/bin/env bash
# publish-crates.sh — the ordered crates.io publish for the backbone.
# Prereq: `cargo login` (the crates.io API token) ONCE. Then this.
# Order matters: qdecorators (no workspace deps) → world-core → ternary
#   → corelib (the facade). Each publish verifies + publishes alone.
set -euo pipefail
cd "$(dirname "$0")/.."

pub() {
  echo "⟦ publish $1 ⟧"
  cargo package -p "$1" --allow-dirty --no-verify >/dev/null 2>&1 || true
  echo "  packaged: $(ls -t target/package/$1-*crate 2>/dev/null | head -1)"
  cargo publish -p "$1"
}

pub qdecorators
pub world-core
pub ternary
pub corelib
echo "⟦ the backbone is on crates.io ⟧"

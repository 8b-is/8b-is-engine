#!/usr/bin/env bash
# release.sh — assemble the versioned release artifacts locally.
# CI runs the same steps (see .github/workflows/release.yml) on tag push.
#
#   ./scripts/release.sh            # build release bins + assemble + .dmg
#   ./scripts/release.sh --debug    # skip the release build, use debug bins
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
HERE="$PWD"

VERSION="${VERSION:-$(git describe --tags --always 2>/dev/null || echo dev)}"
MODE="${1:-release}"
if [[ "$MODE" == "--debug" ]]; then PROFILE="debug"; else PROFILE="release"; fi

echo "⟦ vaked release $VERSION ($PROFILE) ⟧"

# 1. the gateway + sidecars
if [[ "$MODE" == "--debug" ]] && [[ -x "$HERE/../vaked-lsp/target/debug/vaked-nats" ]]; then
  BIN_DIR="$HERE/../vaked-lsp/target/debug"
else
  (cd "$HERE/../vaked-lsp" && cargo build --release --bin vaked-lsp --bin vaked-mcp --bin vaked-nats)
  BIN_DIR="$HERE/../vaked-lsp/target/release"
fi

# 2. assemble the bundle
rm -rf dist && mkdir -p dist/VAKED-ENGINE/{bin,docs,floors,assets}
cp "$BIN_DIR"/vaked-{lsp,mcp,nats} dist/VAKED-ENGINE/bin/
cp README.md scaffold.sh swarm.sh sandbox.sh opencode.json dist/VAKED-ENGINE/ 2>/dev/null || true
cp CREDITS.md LICENSE dist/VAKED-ENGINE/ 2>/dev/null || true
cp docs/*.md dist/VAKED-ENGINE/docs/ 2>/dev/null || true
cp proto/* dist/VAKED-ENGINE/ 2>/dev/null || true
cp -r assets/hero.svg assets/mandala.svg assets/sanctuary.svg assets/deity.svg assets/screenshots dist/VAKED-ENGINE/assets/ 2>/dev/null || true
printf 'VAKED ENGINE %s\nbins: vaked-lsp · vaked-mcp · vaked-nats\nsee README.md + docs/integration-guide.md\n' "$VERSION" > dist/VAKED-ENGINE/VERSION

# 3. the .dmg (macOS) / tar.gz (linux)
case "$(uname -s)" in
  Darwin)
    hdiutil create -volname "VAKED ENGINE $VERSION" \
      -srcfolder dist/VAKED-ENGINE -ov \
      "dist/vaked-engine-$VERSION-macos-arm64.dmg" >/dev/null
    tar -czf "dist/vaked-lsp-$VERSION-macos-arm64.tar.gz" -C "$BIN_DIR" vaked-lsp vaked-mcp vaked-nats
    echo "→ dist/vaked-engine-$VERSION-macos-arm64.dmg"
    echo "→ dist/vaked-lsp-$VERSION-macos-arm64.tar.gz"
    ;;
  Linux)
    tar -czf "dist/vaked-engine-$VERSION-linux-x86_64.tar.gz" -C dist/VAKED-ENGINE .
    tar -czf "dist/vaked-lsp-$VERSION-linux-x86_64.tar.gz" -C "$BIN_DIR" vaked-lsp vaked-mcp vaked-nats
    echo "→ dist/vaked-engine-$VERSION-linux-x86_64.tar.gz"
    echo "→ dist/vaked-lsp-$VERSION-linux-x86_64.tar.gz"
    ;;
esac
ls -lh dist/* | tail -3

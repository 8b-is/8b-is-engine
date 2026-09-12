#!/usr/bin/env bash
# enthea — cross-compile for all the constellation's machines.
# Pure stdlib Go means one `go build` per target, no CGO, no vendored deps.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p dist
targets=(
  darwin/arm64 darwin/amd64
  linux/arm64 linux/amd64 linux/386
  windows/amd64 windows/arm64
  freebsd/amd64
)
for t in "${targets[@]}"; do
  os=${t%/*}; arch=${t#*/}
  out="dist/enthea-$os-$arch"
  if [ "$os" = "windows" ]; then out="$out.exe"; fi
  GOOS=$os GOARCH=$arch CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o "$out" .
  echo "✓ $os/$arch -> $out"
done
echo "all targets built."

#!/bin/sh
# enthea — universal installer.
# Works on any Unix (macOS, Debian/Ubuntu, Alpine, Fedora, …) and Windows
# under git-bash/WSL — it ignores brew/apt/apk entirely and drops the static
# binary for your OS/arch into ~/.local/bin.
#
#   curl -fsSL https://raw.githubusercontent.com/8b-is/enthea/main/install.sh | sh
set -eu

# --- pure-ascii wavelike banner: the family on acid ---
banner() {
  TEXT="8b-is   alex,chris,nate,family and p"
  FRAMES=24; ROWS=4; SINE="0 1 1 2 2 2 1 1 0 0 0 0 0 1 1 2 2 2 2 1 1 0 0 0 0 0 1 1 2 2 2 1"
  set -- $SINE
  len=${#TEXT}; frame=0
  while [ "$frame" -lt "$FRAMES" ]; do
    row=0
    while [ "$row" -lt "$ROWS" ]; do
      line=""; i=0
      while [ "$i" -lt "$len" ]; do
        idx=$(((frame + i) % 32 + 1)); eval "off=\${$idx}"
        if [ "$off" -eq "$row" ]; then line="${line}$(printf '%s' "$TEXT" | cut -c$((i + 1)))"
        else line="${line} "; fi
        i=$((i + 1))
      done
      printf '%s\n' "$line"; row=$((row + 1))
    done
    printf '\n'; frame=$((frame + 1)); sleep 0.04
  done
  printf '%s\n' "   vaked.dev"
}
banner

VERSION="${ENTHEA_VERSION:-latest}"
DEST="${ENTHEA_DEST:-$HOME/.local/bin}"
if [ "$VERSION" = latest ]; then
  BASE="https://github.com/8b-is/enthea/releases/latest/download"
else
  BASE="https://github.com/8b-is/enthea/releases/download/${VERSION}"
fi

os=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$os" in
  darwin) os=darwin ;;
  linux)  os=linux ;;
  msys*|mingw*|cygwin*) os=windows ;;
  *) echo "enthea: unsupported os: $os" >&2; exit 1 ;;
esac

arch=$(uname -m)
case "$arch" in
  x86_64|amd64) arch=amd64 ;;
  aarch64|arm64) arch=arm64 ;;
  i386|i686) arch=386 ;;
  *) echo "enthea: unsupported arch: $arch" >&2; exit 1 ;;
esac

file="enthea-${os}-${arch}"
[ "$os" = windows ] && file="${file}.exe"
url="$BASE/$file"

echo "enthea ${VERSION} (${os}/${arch}) -> ${DEST}"
mkdir -p "$DEST"
curl -fsSL "$url" -o "$DEST/enthea"
chmod +x "$DEST/enthea"

echo
echo "enthea installed to $DEST/enthea"
case ":$PATH:" in
  *":$DEST:"*) ;;
  *) echo "add it to your PATH:  export PATH=\"$DEST:\$PATH\"" ;;
esac
echo "then: enthea setup opencode   # wire MCP + personas into your client"

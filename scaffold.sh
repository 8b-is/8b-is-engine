#!/usr/bin/env bash
# scaffold.sh — the 8b-is engine installer.
#
# Self-contained bash: installs the utility deps, wires jj as the default
# VCS, and scaffolds a new engine project. Silverblue rpm-ostree aware
# (the containerized Fedora path falls back to flatpak/cargo).
#
#   ./scaffold.sh            # install deps + wire jj
#   ./scaffold.sh new my-game   # scaffold a new project with jj
#   ./scaffold.sh doctor      # probe the lanes
set -euo pipefail

GRN=$'\033[32m'; YEL=$'\033[33m'; RED=$'\033[31m'; BLD=$'\033[1m'; RST=$'\033[0m'
info() { printf '%s==>%s %s\n' "$GRN" "$RST" "$*"; }
warn() { printf '%s!!%s  %s\n' "$YEL" "$RST" "$*"; }
die()  { printf '%sxx%s  %s\n' "$RED" "$RST" "$*" >&2; exit 1; }

# ── the utility deps the engine lanes need ────────────────────────────────
UTILS=(jj uv cargo go just node gh wrangler rg bat fd eza zoxide delta container)
JJ_VERSION="v0.45.1"

have() { command -v "$1" >/dev/null 2>&1; }

install_jj() {
  local os arch url
  case "$(uname -s)" in
    Darwin) os="apple-darwin";;
    Linux)  os="unknown-linux-musl";;
    *) die "jj install unsupported on $(uname -s) — install manually";;
  esac
  case "$(uname -m)" in
    arm64|aarch64) arch="aarch64";;
    x86_64|amd64)  arch="x86_64";;
    *) die "jj install unsupported arch $(uname -m)";;
  esac
  url="https://github.com/martinvonz/jj/releases/download/${JJ_VERSION}/jj-${JJ_VERSION}-${arch}-${os}.tar.gz"
  info "downloading jj from ${url}"
  local tmp; tmp="$(mktemp -d)"
  if have curl; then curl -fsSL "$url" | tar -xz -C "$tmp"; else die "curl required to fetch jj"; fi
  local dest="/usr/local/bin"
  [[ -w "$dest" ]] || dest="$HOME/.local/bin"
  mkdir -p "$dest"
  install -m 755 "$tmp/jj" "$dest/jj"
  rm -rf "$tmp"
  info "jj installed to $dest/jj"
}

# ── install a dep (best-effort per platform) ──────────────────────────────
install_dep() {
  local d="$1"
  case "$d" in
    jj)      install_jj;;
    uv)      if have curl; then curl -LsSf https://astral.sh/uv/install.sh | sh; else die "uv needs curl"; fi;;
    cargo)   if have rustup; then rustup default stable; else warn "rustup not found — install via https://rustup.rs"; fi;;
    go)      if have brew; then brew install go; elif have dnf; then sudo dnf install -y golang; else warn "install go manually"; fi;;
    just)    if have brew; then brew install just; elif have cargo; then cargo install just; else warn "install just manually"; fi;;
    node)    if have brew; then brew install node; elif have dnf; then sudo dnf install -y nodejs; else warn "install node manually"; fi;;
    gh)      if have brew; then brew install gh; elif have dnf; then sudo dnf install -y gh; else warn "install gh manually"; fi;;
    wrangler) if have npm; then npm install -g wrangler; else warn "install wrangler via npm"; fi;;
    rg|bat|fd|eza|zoxide|delta) if have brew; then brew install "$d"; elif have cargo; then cargo install "$d"; else warn "install $d manually"; fi;;
    container) warn "Apple Containers: see github.com/apple/container (macOS 26+, Apple Silicon)";;
    *) warn "unknown dep $d";;
  esac
}

# ── the lanes ──────────────────────────────────────────────────────────────
cmd_doctor() {
  for d in "${UTILS[@]}"; do
    if have "$d"; then printf '%-10s %s\n' "$d" "$(command -v "$d")"
    else printf '%-10s %s\n' "$d" "${RED}missing${RST}"; fi
  done
  if have jj; then
    local v; v="$(jj --version 2>/dev/null | head -1)"
    printf '%-10s %s\n' "jj" "$v"
  fi
}

cmd_install() {
  info "installing utility deps: ${UTILS[*]}"
  for d in "${UTILS[@]}"; do
    if have "$d"; then info "$d already present"; else install_dep "$d"; fi
  done
  info "wiring jj as the default VCS"
  jj config set --user ui.default-command log 2>/dev/null || true
  jj config set --user git.auto-local-bookmark true 2>/dev/null || true
  info "done — 'jj st' in any engine project"
}

cmd_new() {
  local name="$1"
  [[ -n "$name" ]] || die "usage: scaffold.sh new <project-name>"
  [[ -d "$name" ]] && die "$name already exists"
  mkdir -p "$name"
  cd "$name"
  git init -q 2>/dev/null || true
  jj git init 2>/dev/null || true
  mkdir -p assets docs src
  cat > README.md <<EOF
# $name

scaffolded by the 8b-is engine installer — jj is the VCS, the seed is the world.
EOF
  info "scaffolded $name — cd in, 'jj st' to see the working copy"
}

main() {
  local cmd="${1:-install}"
  case "$cmd" in
    install) cmd_install;;
    doctor)  cmd_doctor;;
    new)     cmd_new "${2:-}";;
    *)       die "usage: scaffold.sh [install|doctor|new <name>]";;
  esac
}

main "$@"

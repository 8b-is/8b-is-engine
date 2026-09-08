#!/usr/bin/env bash
# scaffold.sh — the 8b-is engine installer + full E2E bootstrap.
#
# Self-contained bash. Gets a new developer from zero to a running world:
# deps → jj → git-lfs → NATS mesh → a live NPC actor → the export lane →
# the creative swarm. Silverblue rpm-ostree aware.
#
#   ./scaffold.sh                 # == bootstrap (the full E2E)
#   ./scaffold.sh install         # deps + jj + lfs
#   ./scaffold.sh verify          # probe every lane (the full doctor)
#   ./scaffold.sh new my-game     # scaffold a jj-backed project
#   ./scaffold.sh mesh            # start nats-server + run the NPC actor demo
#   ./scaffold.sh export "brief"  # seed → Blender EEVEE render
set -euo pipefail

GRN=$'\033[32m'; YEL=$'\033[33m'; RED=$'\033[31m'; MAG=$'\033[35m'; RST=$'\033[0m'
info() { printf '%s==>%s %s\n' "$GRN" "$RST" "$*"; }
warn() { printf '%s!!%s  %s\n' "$YEL" "$RST" "$*"; }
die()  { printf '%sxx%s  %s\n' "$RED" "$RST" "$*" >&2; exit 1; }

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CENTF="$HERE/../centerfugeq"

# ── the utility deps the engine lanes need ────────────────────────────────
UTILS=(jj uv cargo go just node gh wrangler git-lfs nats-server rg bat fd eza zoxide delta container blender hyperfine tokei)
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
    jj)       install_jj;;
    uv)       if have curl; then curl -LsSf https://astral.sh/uv/install.sh | sh; else die "uv needs curl"; fi;;
    cargo)    if have rustup; then rustup default stable; else warn "rustup not found — install via https://rustup.rs"; fi;;
    go)       if have brew; then brew install go; elif have dnf; then sudo dnf install -y golang; else warn "install go manually"; fi;;
    just)     if have brew; then brew install just; elif have cargo; then cargo install just; else warn "install just manually"; fi;;
    node)     if have brew; then brew install node; elif have dnf; then sudo dnf install -y nodejs; else warn "install node manually"; fi;;
    gh)       if have brew; then brew install gh; elif have dnf; then sudo dnf install -y gh; else warn "install gh manually"; fi;;
    git-lfs)  if have brew; then brew install git-lfs; else warn "install git-lfs manually"; fi;;
    nats-server) if have brew; then brew install nats-server; elif have go; then go install github.com/nats-io/nats-server/v2@latest; else warn "install nats-server manually"; fi;;
    wrangler) if have npm; then npm install -g wrangler; else warn "install wrangler via npm"; fi;;
    blender)  if have brew; then brew install --cask blender; else warn "install Blender manually (the export lane needs it)"; fi;;
    rg|bat|fd|eza|zoxide|delta|hyperfine|tokei) if have brew; then brew install "$d"; elif have cargo; then cargo install "$d"; else warn "install $d manually"; fi;;
    container) warn "Apple Containers: see github.com/apple/container (macOS 26+, Apple Silicon)";;
    *) warn "unknown dep $d";;
  esac
}

# ── the lanes ──────────────────────────────────────────────────────────────
cmd_doctor() {
  printf '%s==>%s the tool lanes\n' "$GRN" "$RST"
  for d in "${UTILS[@]}"; do
    if have "$d"; then printf '  %-12s %s\n' "$d" "$(command -v "$d")"
    else printf '  %-12s %s\n' "$d" "${RED}missing${RST}"; fi
  done
  printf '%s==>%s the engine lanes\n' "$GRN" "$RST"
  # vaked-nats built?
  if [[ -x "$HERE/../vaked-lsp/target/debug/vaked-nats" ]]; then echo "  vaked-nats   built (the actor-mesh sidecar)"; else echo "  vaked-nats   not built — cd ../vaked-lsp && cargo build --bin vaked-nats"; fi
  # NATS running?
  if lsof -iTCP:4222 -sTCP:LISTEN >/dev/null 2>&1; then echo "  nats-server  running on :4222"; else echo "  nats-server  down — ./scaffold.sh mesh"; fi
  if lsof -iTCP:9222 -sTCP:LISTEN >/dev/null 2>&1; then echo "  nats-ws      running on ws://:9222 — open client/gaia-dashboard.html"; else echo "  nats-ws      down — ./scaffold.sh mesh (the browser's door to the mesh)"; fi
  # git-lfs
  if have git-lfs; then echo "  git-lfs     $(git lfs version 2>/dev/null | head -1)"; fi
  # the swarm config
  if [[ -f "$HERE/opencode.json" ]]; then echo "  swarm       5 agents (opencode.json, DeepSeek V4 vision)"; fi
  # asset packs
  if ls "$HERE"/assets/vendor/kenney/*.zip >/dev/null 2>&1; then echo "  assets      $(ls "$HERE"/assets/vendor/kenney/*.zip | wc -l | tr -d ' ') CC0 packs vendored"; else echo "  assets      none vendored"; fi
  # centerfugeq export lane
  if [[ -f "$CENTF/quantTernEngine/gen.ts" ]] && grep -q "'scene'" "$CENTF/quantTernEngine/gen.ts"; then echo "  export-lane scene modality ready"; else echo "  export-lane missing scene modality"; fi
}

cmd_install() {
  info "installing utility deps: ${UTILS[*]}"
  for d in "${UTILS[@]}"; do
    if have "$d"; then :; else install_dep "$d"; fi
  done
  info "wiring jj as the default VCS"
  jj config set --user user.name "${GIT_AUTHOR_NAME:-Péter Lódri}" 2>/dev/null || true
  jj config set --user user.email "${GIT_AUTHOR_EMAIL:-cabotage@pm.me}" 2>/dev/null || true
  jj config set --user ui.default-command log 2>/dev/null || true
  jj config set --user git.auto-local-bookmark true 2>/dev/null || true
  info "wiring git-lfs for large assets"
  if have git-lfs; then
    cd "$HERE" && git lfs install 2>/dev/null || true
    if ! grep -q 'filter=lfs' .gitattributes 2>/dev/null; then
      printf '*.png filter=lfs diff=lfs merge=lfs -text\n*.zip filter=lfs diff=lfs merge=lfs -text\n*.wav filter=lfs diff=lfs merge=lfs -text\nassets/vendor/** filter=lfs diff=lfs merge=lfs -text\n' > .gitattributes
    fi
  fi
  info "building the vaked-nats sidecar (actor-mesh)"
  if [[ -d "$HERE/../vaked-lsp" ]]; then
    (cd "$HERE/../vaked-lsp" && cargo build --bin vaked-nats 2>/dev/null && echo "  vaked-nats built") || warn "vaked-nats build skipped (cargo busy?) — retry later"
  fi
  info "done — run ./scaffold.sh verify to probe every lane"
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

cmd_mesh() {
  info "starting nats-server on :4222 + ws :9222 (if down)"
  if ! lsof -iTCP:4222 -sTCP:LISTEN >/dev/null 2>&1; then
    cat > /tmp/nats-ws.conf <<EOF
port: 4222
websocket {
  port: 9222
  no_tls: true
}
EOF
    nats-server -c /tmp/nats-ws.conf >/tmp/nats-server.log 2>&1 &
    sleep 1
    echo "  nats-server pid $! — ws://127.0.0.1:9222 (open client/gaia-dashboard.html)"
  fi
  info "running the needs/goals NPC actor (the world runs without you)"
  if command -v uv >/dev/null 2>&1; then
    uv run --project "$HERE" --with nats-py python "$HERE/examples/mesh-npc.py" --name ལྷ --seed 42 --ticks 40 2>&1 | tail -20
  else
    warn "uv not installed — install then retry (or run mesh-npc.py manually)"
  fi
}

cmd_export() {
  local brief="${1:-the sanctuary at dawn, the 108 gates}"
  if ! have blender; then warn "blender not installed — install it for the export lane"; return; fi
  if [[ ! -f "$CENTF/quantTernEngine/gen.ts" ]]; then warn "centerfugeq not at $CENTF"; return; fi
  info "export lane: seed → manifest → scene.py → Blender EEVEE"
  cd "$CENTF"
  local m
  m="$(node quantTernEngine/gen.ts scene "$brief" 2>/dev/null | grep manifest | awk '{print $2}')"
  [[ -n "$m" ]] || { warn "gen.ts scene failed"; return; }
  node blender3d/scene_builder.ts "$m" out/e2e.scene.py
  ADMISSIBILITY_OUTPUT="$CENTF/out/e2e-export.png" WIDTH=1280 HEIGHT=720 timeout 120 blender --background --python out/e2e.scene.py >/dev/null 2>&1 && echo "  rendered → out/e2e-export.png" || warn "blender render failed"
}

cmd_bootstrap() {
  info "the full E2E bootstrap"
  cmd_install
  cmd_verify
  info "bootstrap complete. day-1:"
  echo "    jj st                       # the working copy"
  echo "    ./scaffold.sh mesh          # NATS + a live NPC actor"
  echo "    ./scaffold.sh export \"...\"  # seed → Blender render"
  echo "    ./swarm.sh \"a brief\"        # the 5-creative-agent fanout"
}

cmd_verify() {
  cmd_doctor
}

main() {
  local cmd="${1:-bootstrap}"
  shift || true
  case "$cmd" in
    install)   cmd_install;;
    doctor|verify) cmd_doctor;;
    new)       cmd_new "${1:-}";;
    mesh)      cmd_mesh;;
    export)    cmd_export "${1:-}";;
    bootstrap) cmd_bootstrap;;
    *)         die "usage: scaffold.sh [bootstrap|install|verify|new <name>|mesh|export \"brief\"]";;
  esac
}

main "$@"

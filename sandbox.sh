#!/usr/bin/env bash
# sandbox.sh — cross-platform dev-tool sandbox orchestrator.
#
# macOS:  Apple Containers (Virtualization.framework, one VM per container)
# Linux:  bubblewrap (LSP/binary wrapping) + podman (full toolchain images)
#
#   sandbox.sh run <image> [cmd...]     run a command in a sandbox
#   sandbox.sh lsp <server>             wrap an LSP server (bwrap on Linux, container on macOS)
#   sandbox.sh build <toolchain> [cmd]  reproducible build in a pinned image
#   sandbox.sh list                     list running sandboxes
#   sandbox.sh stop <name>              stop a named sandbox
#   sandbox.sh status                   probe the sandbox backends
#   sandbox.sh doctor                   check all backends are available
set -euo pipefail

GRN=$'\033[32m'; YEL=$'\033[33m'; RED=$'\033[31m'; BLD=$'\033[1m'; RST=$'\033[0m'
info() { printf '%s==>%s %s\n' "$GRN" "$RST" "$*"; }
warn() { printf '%s!!%s  %s\n' "$YEL" "$RST" "$*"; }
die()  { printf '%sxx%s  %s\n' "$RED" "$RST" "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

PLATFORM="$(uname -s)"
PROJECT_DIR="${SANDBOX_PROJECT:-$PWD}"
SANDBOX_NAME="${SANDBOX_NAME:-$(basename "$PROJECT_DIR")-sandbox}"

# ── macOS: Apple Containers ───────────────────────────────────────────────
mac_container_run() {
  local image="$1"; shift
  info "Apple Container: $image"
  container run --rm -it \
    -v "$PROJECT_DIR:/workspace" \
    -w /workspace \
    --memory "${SANDBOX_MEMORY:-2g}" \
    --cpus "${SANDBOX_CPUS:-4}" \
    "$image" "$@"
}

mac_container_lsp() {
  local server="$1"
  local image="${SANDBOX_LSP_IMAGE:-docker.io/library/rust:latest}"
  info "Apple Container LSP: $server via $image"
  container run --rm -i \
    -v "$PROJECT_DIR:/workspace" \
    -w /workspace \
    --memory 1g --cpus 2 \
    "$image" "$server"
}

mac_container_build() {
  local toolchain="$1"; shift
  local image="${SANDBOX_BUILD_IMAGE:-$toolchain}"
  info "Apple Container build: $image"
  container run --rm -it \
    -v "$PROJECT_DIR:/workspace" \
    -w /workspace \
    --memory "${SANDBOX_MEMORY:-4g}" \
    --cpus "${SANDBOX_CPUS:-4}" \
    "$image" "$@"
}

mac_list()   { container ls 2>/dev/null || info "no containers running"; }
mac_stop()   { container stop "$1" 2>/dev/null || warn "$1 not running"; }
mac_status() {
  if have container; then
    info "Apple Containers: $(container system version 2>/dev/null | head -1 || echo 'installed')"
    container system status 2>/dev/null || true
  else
    warn "Apple Containers not installed — brew install apple/container or see github.com/apple/container"
  fi
}

# ── Linux: bubblewrap (LSP) + podman (build) ─────────────────────────────
linux_bwrap_lsp() {
  local server="$1"
  info "bubblewrap LSP: $server (read-only toolchain, writable project)"
  bwrap \
    --unshare-all --share-net \
    --new-session --die-with-parent \
    --ro-bind /usr /usr \
    --ro-bind-try /lib /lib \
    --ro-bind-try /lib64 /lib64 \
    --symlink usr/bin /bin \
    --proc /proc --dev /dev --tmpfs /tmp \
    --bind "$PROJECT_DIR" "$PROJECT_DIR" \
    --chdir "$PROJECT_DIR" \
    "$server"
}

linux_podman_run() {
  local image="$1"; shift
  info "podman: $image"
  podman run --rm -it --userns=keep-id \
    -v "$PROJECT_DIR:/workspace:Z" \
    -w /workspace \
    --cap-drop ALL \
    "$image" "$@"
}

linux_podman_build() {
  local toolchain="$1"; shift
  local image="${SANDBOX_BUILD_IMAGE:-$toolchain}"
  info "podman build: $image"
  podman run --rm -it --userns=keep-id \
    -v "$PROJECT_DIR:/workspace:Z" \
    -w /workspace \
    --memory "${SANDBOX_MEMORY:-4g}" \
    --cpus "${SANDBOX_CPUS:-4}" \
    --cap-drop ALL \
    "$image" "$@"
}

linux_list()   { podman ps 2>/dev/null || info "no containers running"; }
linux_stop()   { podman stop "$1" 2>/dev/null || warn "$1 not running"; }
linux_status() {
  if have bwrap; then info "bubblewrap: $(bwrap --version 2>/dev/null | head -1 || echo 'installed')"; else warn "bubblewrap not installed"; fi
  if have podman; then info "podman: $(podman --version 2>/dev/null || echo 'installed')"; else warn "podman not installed"; fi
}

# ── cross-platform dispatch ───────────────────────────────────────────────
cmd_run() {
  local image="${1:?usage: sandbox.sh run <image> [cmd...]}"; shift
  case "$PLATFORM" in
    Darwin) mac_container_run "$image" "$@";;
    Linux)  linux_podman_run "$image" "$@";;
    *)      die "unsupported platform: $PLATFORM";;
  esac
}

cmd_lsp() {
  local server="${1:?usage: sandbox.sh lsp <server>}"; shift
  case "$PLATFORM" in
    Darwin) mac_container_lsp "$server";;
    Linux)  linux_bwrap_lsp "$server";;
    *)      die "unsupported platform: $PLATFORM";;
  esac
}

cmd_build() {
  local toolchain="${1:?usage: sandbox.sh build <toolchain> [cmd...]}"; shift
  case "$PLATFORM" in
    Darwin) mac_container_build "$toolchain" "$@";;
    Linux)  linux_podman_build "$toolchain" "$@";;
    *)      die "unsupported platform: $PLATFORM";;
  esac
}

cmd_list() {
  case "$PLATFORM" in
    Darwin) mac_list;;
    Linux)  linux_list;;
  esac
}

cmd_stop() {
  local name="${1:?usage: sandbox.sh stop <name>}"
  case "$PLATFORM" in
    Darwin) mac_stop "$name";;
    Linux)  linux_stop "$name";;
  esac
}

cmd_status() {
  info "platform: $PLATFORM · project: $PROJECT_DIR"
  case "$PLATFORM" in
    Darwin) mac_status;;
    Linux)  linux_status;;
  esac
}

cmd_doctor() {
  info "sandbox backends:"
  case "$PLATFORM" in
    Darwin)
      have container && info "  Apple Containers: $(container system version 2>/dev/null | head -1)" || warn "  Apple Containers: MISSING (brew install apple/container)"
      ;;
    Linux)
      have bwrap  && info "  bubblewrap: $(bwrap --version 2>/dev/null | head -1)" || warn "  bubblewrap: MISSING (apt/dnf/pacman install bubblewrap)"
      have podman && info "  podman: $(podman --version 2>/dev/null)" || warn "  podman: MISSING (apt/dnf/pacman install podman)"
      ;;
  esac
  info "env overrides: SANDBOX_PROJECT SANDBOX_NAME SANDBOX_MEMORY SANDBOX_CPUS SANDBOX_LSP_IMAGE SANDBOX_BUILD_IMAGE"
}

main() {
  local cmd="${1:-status}"; shift || true
  case "$cmd" in
    run)    cmd_run "$@";;
    lsp)    cmd_lsp "$@";;
    build)  cmd_build "$@";;
    list)   cmd_list;;
    stop)   cmd_stop "$@";;
    status) cmd_status;;
    doctor) cmd_doctor;;
    *)      die "usage: sandbox.sh [run|lsp|build|list|stop|status|doctor]";;
  esac
}

main "$@"

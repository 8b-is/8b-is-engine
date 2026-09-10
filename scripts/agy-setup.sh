#!/usr/bin/env bash
# agy-setup.sh — download + install the local brain (the Antigravity /
# Gemini CLI), let the USER navigate the first auth, then reuse the
# session. OAuth expires every 4-6h: `ensure` probes cheaply and tells
# you the ONE command to re-honor — then it is silent again.
#
#   agy-setup install    # download + install the gemini CLI
#   agy-setup auth       # the first auth — the USER drives the browser
#   agy-setup ensure     # probe the session; if expired, say how to fix
#   agy-setup api-key    # the zero-babysitting path: pin GEMINI_API_KEY

set -euo pipefail

BIN="/opt/homebrew/bin"
if [[ ! -d "$BIN" ]]; then BIN="$HOME/.local/bin"; fi
CLI="$BIN/gemini"
LATEST="0.46.0" # pinned — bump with a release; brew tracks the freshest

have() { command -v "$1" >/dev/null 2>&1; }

cmd_install() {
  if have gemini; then
    local v; v="$(gemini --version 2>/dev/null | head -1)"
    echo "agy: gemini already installed ($v) at $(command -v gemini)"
    [[ "$v" < "$LATEST" ]] && echo "agy: newer releases exist — brew upgrade gemini"
  elif have brew; then
    echo "agy: installing via brew (the m1-friendly path)"
    brew install gemini
  else
    echo "agy: downloading the darwin-arm64 tarball → $BIN"
    local url="https://github.com/google-gemini/gemini-cli/releases/latest/download/gemini-darwin-arm64.zip"
    local tmp; tmp="$(mktemp -d)"
    curl -fsSL "$url" -o "$tmp/gemini.zip"
    (cd "$tmp" && unzip -q gemini.zip)
    mkdir -p "$BIN"
    install -m 755 "$tmp"/gemini* "$BIN/gemini" 2>/dev/null || install -m 755 "$tmp"/gemini "$BIN/gemini"
    rm -rf "$tmp"
  fi
  echo "  → scripts/agy \"…\"  (the wrapper waits in the heap)"
}

cmd_auth() {
  have gemini || { echo "agy: install first — agy-setup install" >&2; exit 1; }
  echo "⟦ first auth ⟧ the browser flow is YOURS — I just relay it"
  echo "  prefer the zero-expiry path?  export GEMINI_API_KEY=… then: agy-setup api-key"
  echo "  else run: gemini -p \"auth ping\" -m gemini-3-flash"
  echo "           (it prints the google URL + code; paste the code back)"
  echo "  … waiting for you to complete it — then this session persists (~/.gemini)"
  gemini -p "auth ping" -m gemini-3-flash
}

cmd_ensure() {
  have gemini || { echo "agy: not installed — agy-setup install" >&2; exit 1; }
  if [[ -n "${GEMINI_API_KEY:-}" ]]; then echo "agy: on the api-key path — no expiry to fear"; exit 0; fi
  # a cheap headless probe: 3-flash answers "ok" fast when the session is hot
  if timeout 25 gemini -p "ok" -m gemini-3-flash >/dev/null 2>&1; then
    echo "agy: the session is hot — reusing ~/.gemini (creds persist; expiry ~4-6h)"
    exit 0
  fi
  echo "agy: the OAuth session expired (~4-6h lifetime) — ONE command re-honors it:"
  echo "  agy-setup auth"
  exit 1
}

cmd_api_key() {
  have gemini || { echo "agy: install first — agy-setup install" >&2; exit 1; }
  [[ -n "${GEMINI_API_KEY:-}" ]] || { echo "agy: export GEMINI_API_KEY first (ai.google.dev)" >&2; exit 1; }
  echo "agy: api-key pinned — the wrapper uses it (GEMINI_API_KEY), no periodic auth"
  exit 0
}

case "${1:-}" in
  install) cmd_install;;
  auth)    cmd_auth;;
  ensure)  cmd_ensure;;
  api-key) cmd_api_key;;
  *) echo "usage: agy-setup [install|auth|ensure|api-key]" >&2; exit 2;;
esac

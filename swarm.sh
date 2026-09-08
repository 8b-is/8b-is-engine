#!/usr/bin/env bash
# swarm.sh — the creative-subagent fanout for the 8b-is engine.
#
# Fans one brief out to the five creative roles (game-art · game-design ·
# frontend · ui · visual-artist), each running as an opencode subagent on
# the DeepSeek V4 model. Every agent writes its artifact to out/swarm/<role>/.
#
#   ./swarm.sh "the painted-forest dawn zone, ZEN mechanic"
#   ./swarm.sh --roles visual-artist,ui "the pink tent HUD"
#   ./swarm.sh --model deepseek/deepseek-v4-pro "the sanctuary vertical slice"
#
# KV-cache note: all five agent prompts in opencode.json share one
# byte-identical prefix (the doctrine + palette + docs block), so DeepSeek's
# automatic context caching serves the shared prefix as a cache hit once the
# first agent has run — check usage.prompt_cache_hit_tokens in the API reply.
set -euo pipefail

GRN=$'\033[32m'; YEL=$'\033[33m'; MAG=$'\033[35m'; CYN=$'\033[36m'; RST=$'\033[0m'
ROLES=(game-art game-design frontend ui visual-artist)
MODEL="${SWARM_MODEL:-deepseek/deepseek-v4-flash}"
BRIEF=""
OUT="out/swarm"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --roles) IFS=',' read -r -a ROLES <<< "$2"; shift 2;;
    --model) MODEL="$2"; shift 2;;
    -h|--help) echo "usage: ./swarm.sh [--roles a,b,c] [--model provider/model] \"the brief\""; exit 0;;
    *) BRIEF="$BRIEF $1"; shift;;
  esac
done
BRIEF="$(echo "$BRIEF" | xargs)"
[[ -n "$BRIEF" ]] || { echo "give the swarm a brief"; exit 1; }

command -v opencode >/dev/null 2>&1 || { echo "opencode not installed"; exit 1; }
mkdir -p "$OUT"

echo "⟦ the creative swarm ⟧ ${#ROLES[@]} roles · $MODEL"
echo "brief: $BRIEF"
echo

PIDS=()
for role in "${ROLES[@]}"; do
  DIR="$OUT/$role"
  mkdir -p "$DIR"
  printf '%s==>%s %s → %s/ (opencode run --agent %s)\n' "$MAG" "$RST" "$role" "$DIR" "$role"
  opencode run --agent "$role" --model "$MODEL" \
    "Work in $DIR. Create/update the artifact(s) for this brief — write real files (markdown specs, code, or SVG), keep them self-contained, and follow your role prompt in opencode.json.

BRIEF: $BRIEF

When done, write a 2-3 line summary to $DIR/SUMMARY.md." > "$DIR/run.log" 2>&1 &
  PIDS+=("$!")
done

FAIL=0
for i in "${!PIDS[@]}"; do
  role="${ROLES[$i]}"
  if wait "${PIDS[$i]}"; then
    printf '%s✓ %s%s finished\n' "$GRN" "$role" "$RST"
  else
    printf '%s✗ %s%s failed — see %s/%s/run.log\n' "$YEL" "$role" "$RST" "$OUT" "$role"
    FAIL=1
  fi
done

echo
if [[ $FAIL -eq 0 ]]; then
  echo "⟦ the swarm returns ⟧ artifacts under $OUT/"
  find "$OUT" -type f | sort
else
  echo "some roles failed — check their run.logs"
fi

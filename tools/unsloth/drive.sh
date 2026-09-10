#!/usr/bin/env bash
# drive.sh — the obliterated lane's headless driver (against the Colab
# CLI, no browser after the one-time auth): provision the VM, upload the
# world set, run the fine-tune, download the exports, seat the GGUF in
# the sidecar mirror. The corridor: docs/obliterated-lane.md.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
S="${S:-lane}"
MODEL="${MODEL:-OBLITERATUS/Ornith-1.5-9B-OBLITERATED}"
STEPS="${STEPS:-120}"

# gate: the auth must already be cached (run tools/unsloth/auth.sh once)
if ! colab whoami >/dev/null 2>&1; then
  echo "⟦ gate ⟧ run tools/unsloth/auth.sh once (the browser hand-off), then re-run"
  exit 1
fi

echo "⟦ 1 · the runtime ⟧ $S (L4)"
colab new --gpu L4 --keep -s "$S"

echo "⟦ 2 · the world set ⟧ /content/world_dataset.jsonl"
colab upload -s "$S" "$HERE/out/world_dataset.jsonl" /content/world_dataset.jsonl
colab upload -s "$S" "$HERE/../../assets/ternary/corpus.txt" /content/corpus.txt

echo "⟦ 3 · the fine-tune ⟧ ${MODEL} · ${STEPS} steps"
MODEL="$MODEL" STEPS="$STEPS" WORLD=/content/world_dataset.jsonl \
  colab exec -s "$S" -f "$HERE/run_finetune.py"

echo "⟦ 4 · the exports ⟧"
mkdir -p "$HERE/out/finetune"
colab download -s "$S" /content/out/merged_fp16 "$HERE/out/finetune/" -r 2>/dev/null \
  || colab download -s "$S" /content/out "$HERE/out/finetune/" -r
colab ls -s "$S" /content/out/gguf || true

echo "⟦ 5 · the mirror (10 GB leash) ⟧"
GGUF=$(find "$HERE/out/finetune" -name '*.gguf' | head -1)
if [[ -n "$GGUF" ]]; then
  uv run "$HERE/../mirror/mirror.py" add "$GGUF" --kind model --source "the obliterated lane"
  uv run "$HERE/../mirror/mirror.py" index
fi

echo "⟦ done ⟧ the obliterated lane speaks — stop the VM with: colab stop -s $S (or leave --keep)"

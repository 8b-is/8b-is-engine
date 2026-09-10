# the obliterated lane — unsloth · mlx · gguf on the top-4

*Brainstormed with the founder (2026-09-10). The constellation takes the
four most popular abliterated models and makes them its own: QLoRA
fine-tune with unsloth, exported into BOTH native formats the fleet
speaks — MLX for the M-series workstations, GGUF for the shelf and the
mirror. The fine-tune is a voice transplant: the models keep their
abliterated openness, and the constellation's corpus teaches them to
speak like the world.*

## the model matrix (hardware-friendly top-4)

| model | base | Q4 size | lane | where it runs |
|---|---|---|---|---|
| `Ornith-1.5-9B-OBLITERATED` | Qwen3.5 hybrid (Gated DeltaNet) | ~5.5 GiB | unsloth fine-tune → GGUF + MLX | the headline: Colab T4/A100 + the Mac |
| `gemma-4-E4B-it-OBLITERATED` | Gemma 4, ~4B active | ~2.2 GiB | MLX local + GGUF | the always-on room |
| `Gemma-4-12B-OBLITERATED` | Gemma 4 12B | ~7 GiB | Colab → GGUF | the deep room |
| `Qwen3.8-27B-OBLITERATED` | Qwen 3.8 27B | ~16 GiB | later (not on T4, not in the mirror) | the far room |

The **10 GB mirror cap** is honest math: E4B + Ornith Q4 = ~7.7 GiB
fits; Gemma-12B (7 GiB) enters only by LRU-pruning one of them, and the
27B stays out until the cap is raised. The mirror's `prune --max 10G`
enforces it.

## the three lanes

### lane 1 · unsloth (the train)

`tools/unsloth/colab_finetune.ipynb` — the notebook draft: unsloth
`UnslothTrainer` QLoRA at 4-bit, `max_seq_length 4096`, the dataset is
the **mix**: the constellation's world set (built by
`tools/unsloth/build_world_dataset.py` from the doctrine, the floors'
prose, the engrams, the dream corpus) + a public instruct set loaded at
train time. The merged adapter is exported twice: `mlx` (per `mlx-lm`
layout) and `fp16`→GGUF.

- **on Colab**: the mcp-proxy session (the browser window, once) with
  the `peter.lodri@gmail.com` account, T4/A100 runtime — the notebook's
  cells run through the proxy from this workspace.
- **locally on the M3**: unsloth also drives the small models headless
  (`mlx_lm.lora` handles the fine-tune directly on Apple silicon —
  gemma-E4B and Ornith-9B are the ceiling).

### lane 2 · MLX (the Mac, first-class)

`mlx_lm.convert` with 4-bit quantization and group size 64 →
`mlx-community`-style layout under `tools/unsloth/out/mlx/<model>`,
runnable by `mlx_lm.generate` alongside the engine's llama.cpp/MLX
router. The Mac's own fine-tunes land here first (fast iteration).

### lane 3 · GGUF (the shelf + the mirror)

unsloth/`fp16` merge → llama.cpp `convert_hf_to_gguf.py` + an imatrix
pass (`llama-imatrix` on the constellation corpus — the calibration IS
the world) → Q4_K_M / Q5_K_M. Seated into the sidecar mirror:

```bash
uv run tools/mirror/mirror.py add out/gguf/ornith-9b-q4.gguf --kind model --source "the obliterated lane"
uv run tools/mirror/mirror.py index   # the catalogue accounts the 10 GB leash
```

## the wiring (colab-mcp)

`.mcp.json` gains the `colab-mcp` server (`uvx
git+https://github.com/googlecolab/colab-mcp`); the flow:

1. the agent calls `open_colab_browser_connection` → a Colab tab opens
   in the browser (the founder signs in with `peter.lodri@gmail.com`).
2. the notebook-editing tools (add cells, run, `!pip install`,
   upload) arrive through the proxy — the agent drives the notebook
   like a hand.
3. the T4 runtime runs the compact cells: install unsloth, pull the
   model, `!python colab_finetune.py`-style run, export, download the
   merged weights back through the proxy.

Browser auth is the non-headless step (token + origin check; Colab-side
OAuth) — the once-per-session hand-off the founder does gladly.

## the dataset (the world speaks)

`tools/unsloth/build_world_dataset.py` turns the constellation's text
into instruction pairs: the doctrine lines become `{instruction:
"the keeper folds", output: "admissible or refused, with the reason"}`-
shaped turns; the floors' banners become system-voice lines; the
engrams' kinds become labels. The mix with the instruct set happens in
the notebook (HF dataset loaded on top).

## the release ritual on the lane

- fine-tune → eval (perplexity + a dream-penetration-style check) →
  export both formats → mirror seat → the mixed verdict table lands in
  the vault note.
- the plan's honest gates: the 10 GB mirror cap, the T4's model ceiling
  (9B/12B), the browser-auth hand-off.

*the constellation · 0 + 1 · fine touch from within · vaked.dev*

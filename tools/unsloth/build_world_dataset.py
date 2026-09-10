#!/usr/bin/env python3
"""build_world_dataset.py — the world speaks: the constellation's text,
turned into instruction pairs for the obliterated lane's unsloth
fine-tune.

Sources (all local): the sanctuary doctrine (assets/ternary/corpus.txt),
the floors' voice lines (assets/blueprints/*.txt), and the engram kinds.
The mix with a public instruct set happens at train time in the
notebook; this builds the WORLD side — deterministic, seeded, honest.

Usage: uv run tools/unsloth/build_world_dataset.py [--seed 42]
"""

# /// script
# requires-python = ">=3.11"
# ///

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent  # 8b-is-engine/


def corpus_pairs(text: str) -> list[dict]:
    """doctrine lines become instruction/output turns: the line is the
    instruction, the fold's verdict is the output."""
    system = "you are the 8b-is engine: the keeper folds, the world runs without you."
    pairs = []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        pairs.append(
            {
                "system": system,
                "instruction": f"the keeper folds: {line}",
                "output": "admissible — the continuation is kept, and the reason is the document.",
            }
        )
        pairs.append(
            {
                "system": system,
                "instruction": f"what does the world do with: {line}",
                "output": "it keeps it: every attested trace and every durable refusal is written to the ledger.",
            }
        )
    return pairs


def blueprint_pairs() -> list[dict]:
    """the blueprints' title blocks become system-voice turns."""
    system = "you are the constellation's architect, drawing in METSZET's manner."
    pairs = []
    for b in sorted((ROOT / "assets/blueprints").glob("*.txt")):
        title = b.stem.replace("-", " ").title()
        pairs.append(
            {
                "system": system,
                "instruction": f"cut the section of {title}",
                "output": "the section is admissible: it cuts the storeys the way the keeper cuts the ticks.",
            }
        )
    return pairs


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "tools/unsloth/out/world_dataset.jsonl"))
    args = ap.parse_args()

    corpus = (ROOT / "assets/ternary/corpus.txt").read_text()
    pairs = corpus_pairs(corpus) + blueprint_pairs()

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "w", encoding="utf-8") as f:
        for p in pairs:
            f.write(json.dumps(p, ensure_ascii=False) + "\n")

    print(f"⟦ the world speaks ⟧ {len(pairs)} pairs → {out}")
    kinds = {}
    for p in pairs:
        kinds[p["system"][:12]] = kinds.get(p["system"][:12], 0) + 1
    for k, v in sorted(kinds.items()):
        print(f"  {k}…  ×{v}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""dream-eval.py — the dream's penetration, measured (the polars
catalog[dream]).

Penetration answers one question: how deep does the dream reach INTO
the world it was trained on? Three honest metrics over a battery of
seeded dreams (the trainer's own sampler, temperature 0.8):

  * ngram_hit   — fraction of the dream's 3-grams that occur somewhere
                  in the corpus (the dream re-enters the doctrine)
  * vocab_pen   — fraction of the corpus's unique characters the dream
                  manages to speak at least once
  * doctrine    — the longest common substring between the dream and
                  the corpus, normalized by the dream's length (the
                  dream recovers whole doctrine sentences, or not)

The table is stored as dreams.parquet (arrow/parquet) next to the
mirror catalog — the [dream] slice of the polars index.

Usage: uv run tools/dream-eval.py [--seeds N] [--tokens N] [--out dreams.parquet]
"""

# /// script
# requires-python = ">=3.11"
# dependencies = ["polars", "numpy"]
# ///

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ENGINE = ROOT.parent
CORPUS = ENGINE / "assets/ternary/corpus.txt"
TRAINER = ROOT / "train_ternary.py"

SEEDS = [
    "the world runs without",
    "a promise is a promise",
    "the keeper folds",
    "inside the sanctuary walls",
    "the wire is",
    "sometimes he trains",
]


def corpus_ngrams(text: str, n: int) -> set:
    return {text[i : i + n] for i in range(len(text) - n + 1)}


def longest_common_substring(a: str, b: str) -> str:
    if not a or not b:
        return ""
    prev = [0] * (len(b) + 1)
    best_len = 0
    best_end = 0
    for i in range(1, len(a) + 1):
        cur = [0] * (len(b) + 1)
        ai = a[i - 1]
        for j in range(1, len(b) + 1):
            if ai == b[j - 1]:
                cur[j] = prev[j - 1] + 1
                if cur[j] > best_len:
                    best_len = cur[j]
                    best_end = i
            else:
                cur[j] = 0
        prev = cur
    return a[best_end - best_len : best_end]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--seeds", type=int, default=6)
    ap.add_argument("--tokens", type=int, default=120)
    ap.add_argument("--out", default=str(ENGINE / "tools/mirror/dreams.parquet"))
    args = ap.parse_args()

    corpus = CORPUS.read_text()
    trigrams = corpus_ngrams(corpus, 3)
    vocab = set(corpus)

    # the trainer's own sampler — the dream that ships with the world
    sys.path.insert(0, str(TRAINER.parent))
    import importlib.util

    spec = importlib.util.spec_from_file_location("trainer", TRAINER)
    trainer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(trainer)

    import polars as pl

    rows = []
    for i, seed in enumerate(SEEDS[: args.seeds]):
        try:
            dream = trainer.dream_text(
                trainer.TernModel.__new__(trainer.TernModel), "", seed, args.tokens
            )
        except Exception:
            # fall back to the loader path used by --dream (the file
            # reader reconstructs the model from the checkpoint)
            dream = _dream_via_loader(trainer, seed, args.tokens)
        hit = sum(1 for k in range(len(dream) - 2) if dream[k : k + 3] in trigrams)
        ngram_hit = hit / max(1, len(dream) - 2)
        spoken = set(dream)
        vocab_pen = len(spoken & vocab) / max(1, len(vocab))
        lcs = longest_common_substring(dream, corpus)
        doctrine = len(lcs) / max(1, len(dream))
        rows.append(
            {
                "seed": seed,
                "tokens": len(dream),
                "ngram_hit": round(ngram_hit, 4),
                "vocab_pen": round(vocab_pen, 4),
                "doctrine": round(doctrine, 4),
                "lcs_sample": lcs[:40],
            }
        )

    df = pl.DataFrame(rows)
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    df.write_parquet(args.out)
    with pl.Config() as _:
        print(df.select(["seed", "ngram_hit", "vocab_pen", "doctrine"]))
    print(f"⟦ penetration ⟧ {df.height} dreams → {args.out}")
    return 0


def _dream_via_loader(trainer, seed, n):
    import struct

    import numpy as np

    data = (ENGINE / "assets/ternary/sanctuary-1.58.tern").read_bytes()
    v = struct.unpack("<I", data[10:14])[0]
    d = struct.unpack("<I", data[14:18])[0]
    nl = struct.unpack("<I", data[18:22])[0]
    model = trainer.TernModel.__new__(trainer.TernModel)
    model.vocab, model.dim = v, d
    pos = 22
    emb_n = struct.unpack("<Q", data[pos + 1 : pos + 9])[0]
    model.emb = np.frombuffer(data[pos + 9 : pos + 9 + emb_n], "<f4").reshape(v, d).astype(np.float32)
    pos += 9 + emb_n
    model.W = []
    for _ in range(nl):
        plen = struct.unpack("<Q", data[pos + 1 : pos + 9])[0]
        o = struct.unpack("<I", data[pos + 9 : pos + 13])[0]
        g = struct.unpack("<f", data[pos + 13 : pos + 17])[0]
        wl = struct.unpack("<I", data[pos + 17 : pos + 21])[0]
        packed = data[pos + 21 : pos + 21 + wl]
        q = trainer.unpack_trits(packed, o * d).reshape(o, d)
        wl_ = trainer.TriLayer.__new__(trainer.TriLayer)
        wl_.fp = wl_.q = q.astype(np.float32)
        wl_.q16 = q.astype(np.int16)
        wl_.gamma = float(g)
        model.W.append(wl_)
        pos += 9 + plen
    import json

    manifest = json.loads((ENGINE / "assets/ternary/sanctuary-1.58.json").read_text())
    return trainer.dream_text(model, manifest["chars"], seed, n)


if __name__ == "__main__":
    sys.exit(main())

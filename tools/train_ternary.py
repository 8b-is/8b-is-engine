#!/usr/bin/env python3
"""train_ternary.py — the 1.58-bit lane's trainer.

Trains the engine's base model: a tiny BitNet b1.58-style char model whose
forward pass IS the Rust forward pass in `crates/ternary` — the same
quantized i16 activations, the same i32 accumulation, the same single f32
scale — so a checkpoint born here dreams identically on an AMD Zen CPU, an
Apple core, a WASM sandbox, and the GPU lanes.

Usage (uv, the workspace's only Python path):

    uv run tools/train_ternary.py                      # train + write the checkpoint
    uv run tools/train_ternary.py --verify             # re-run the eval path over the corpus
    uv run tools/train_ternary.py --dream "seed" 160   # sample from the written checkpoint

Everything is seeded: the same corpus, the same seed, the same steps, the
same checkpoint — bit for bit.
"""

# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy"]
# ///

import argparse
import hashlib
import json
import struct
import sys
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "assets" / "ternary" / "corpus.txt"
WEIGHTS = ROOT / "assets" / "ternary" / "sanctuary-1.58.tern"
MANIFEST = ROOT / "assets" / "ternary" / "sanctuary-1.58.json"

MAGIC = b"TERN1.58"
REVISION = 1
QUANT_HEADROOM = 16000.0
LN_EPS = 1e-5
SEED = 41592  # the lane's seed: "1 5 8" in decimal
HIDDEN = 128  # dim
N_MID = 2  # residual ternary layers before the head

# ----------------------------------------------------------------------
# the engine's exact math, in numpy
# ----------------------------------------------------------------------


def seed_rng(seed: int) -> np.random.RandomState:
    return np.random.RandomState(seed)


def mulberry32(seed: int):
    """The engine's PRNG in python — mulberry32, bit-identical to
    world-core::tern and the mesh actors' tern.ts.
    """
    a = seed & 0xFFFFFFFF

    def next_float():
        nonlocal a
        a = (a + 0x6D2B79F5) & 0xFFFFFFFF
        t = a
        t = ((t ^ (t >> 15)) * (t | 1)) & 0xFFFFFFFF
        t = (t + ((t ^ (t >> 7)) * (t | 61) ^ t)) & 0xFFFFFFFF
        return ((t ^ (t >> 14)) & 0xFFFFFFFF) / 4294967296.0

    return next_float


def layernorm(x: np.ndarray) -> np.ndarray:
    mean = x.mean()
    var = ((x - mean) ** 2).mean()
    return (x - mean) / np.sqrt(var + LN_EPS)


def quant_acts(x: np.ndarray) -> np.ndarray:
    s = np.abs(x).max() / QUANT_HEADROOM
    return np.clip(np.round(x / s), -QUANT_HEADROOM, QUANT_HEADROOM).astype(np.int16), s


def ternary_linear(w16: np.ndarray, x: np.ndarray, gamma: float) -> np.ndarray:
    """w16: (n_out, n_in) i16 weights of {-1,0,+1}; returns fp32 logits."""
    q, s_x = quant_acts(x)
    acc = (w16.astype(np.int32) @ q.astype(np.int32)).astype(np.float32)
    return acc * s_x * gamma  # the scale exactly once


def tri_state(w: np.ndarray, gamma: float) -> np.ndarray:
    """The BitNet b1.58 recipe: roundclip(w/(gamma+eps)) into {-1,0,+1}."""
    s = np.round(w / (gamma + 1e-5))
    return np.clip(s, -1.0, 1.0)


# ----------------------------------------------------------------------
# the model
# ----------------------------------------------------------------------


class TernModel:
    def __init__(self, vocab: int, seed: int):
        rng = seed_rng(seed)
        self.vocab = vocab
        self.dim = HIDDEN
        self.emb = (rng.rand(vocab, HIDDEN) - 0.5) * 0.2
        # hidden layers: dim→dim; head: dim→vocab
        self.W = [TriLayer(HIDDEN, HIDDEN, rng) for _ in range(N_MID)]
        self.W.append(TriLayer(HIDDEN, vocab, rng))
        for w in self.W:
            w.gamma = np.abs(w.fp).mean()

    def forward_token(self, tok: int, hidden: np.ndarray) -> np.ndarray:
        """One token step — the engine's exact arithmetic order."""
        x = self.emb[tok].copy() + hidden
        for l in range(N_MID):
            ln = layernorm(x)
            y = ternary_linear(self.W[l].q16, ln, self.W[l].gamma)
            x += np.maximum(y, 0.0)
        hidden[:] = x
        ln = layernorm(x)
        return ternary_linear(self.W[N_MID].q16, ln, self.W[N_MID].gamma)

    def params(self) -> int:
        return int(sum(w.q16.size for w in self.W))

    def forward_nograd(self, toks):
        """The eval path: full-sequence forward, returns logits array."""
        hidden = np.zeros(self.dim, dtype=np.float32)
        logits = []
        for t in toks:
            logits.append(self.forward_token(t, hidden))
        return np.asarray(logits)

    def ste(self):
        """Refresh the ternary views from fp with straight-through grad
        wiring (forward sees the tri-state; backward flows to fp)."""
        for w in self.W:
            w.q = tri_state(w.fp, w.gamma)
            w.q16 = w.q.astype(np.int16)


class TriLayer:
    def __init__(self, n_in, n_out, rng):
        scale = 1.0 / np.sqrt(n_in)
        self.fp = (rng.rand(n_out, n_in) - 0.5) * 2.0 * scale
        self.gamma = 1.0
        self.q = np.zeros((n_out, n_in), dtype=np.float32)
        self.q16 = np.zeros((n_out, n_in), dtype=np.int16)


# ----------------------------------------------------------------------
# training — one full-corpus pass per step (deterministic window order),
# Adam with straight-through gradients, gradient clipping. The reported
# CE IS the full-corpus eval CE; `--verify` re-runs the same path and
# must reproduce it bit for bit.
# ----------------------------------------------------------------------


def train(model: TernModel, tokens: np.ndarray, steps: int, lr0: float, window: int):
    xs, ys = tokens[:-1], tokens[1:]
    n = len(xs)
    model.ste()

    # Adam state
    m = [np.zeros_like(w.fp) for w in model.W]
    v = [np.zeros_like(w.fp) for w in model.W]
    m_emb = np.zeros_like(model.emb)
    v_emb = np.zeros_like(model.emb)
    b1, b2, eps = 0.9, 0.999, 1e-8

    ce_history = []
    for step in range(1, steps + 1):
        lr = lr0 * (0.5 + 0.5 * np.cos(np.pi * step / steps))
        total_loss = 0.0
        n_tok = 0
        gW = [np.zeros_like(w.fp) for w in model.W]
        gemb = np.zeros_like(model.emb)

        for i0 in range(0, n - window, window // 2):  # overlapping windows
            seq_x = xs[i0 : i0 + window]
            targets = ys[i0 : i0 + window]
            hidden = np.zeros(model.dim, dtype=np.float32)

            for tok, tgt in zip(seq_x, targets):
                x = model.emb[tok].copy() + hidden
                caches = []
                for l in range(N_MID):
                    ln = layernorm(x)
                    q, s_x = quant_acts(ln)
                    w = model.W[l]
                    acc = (w.q16.astype(np.int32) @ q.astype(np.int32)).astype(np.float32)
                    y = acc * s_x * w.gamma
                    y_act = np.maximum(y, 0.0)
                    caches.append((ln, q, s_x, y, y_act, w))
                    x = x + y_act
                hidden[:] = x
                ln = layernorm(x)
                q, s_x = quant_acts(ln)
                w = model.W[N_MID]
                acc = (w.q16.astype(np.int32) @ q.astype(np.int32)).astype(np.float32)
                logits = acc * s_x * w.gamma

                logits -= logits.max()
                exps = np.exp(logits)
                probs = exps / exps.sum()
                total_loss -= np.log(max(probs[tgt], 1e-12))
                n_tok += 1
                dlogits = probs.copy()
                dlogits[tgt] -= 1.0

                ds = dlogits * s_x * w.gamma
                gW[N_MID] += ds[:, None] * q[None, :]
                dln = w.q16.astype(np.float32).T @ ds
                d0 = dln - dln.mean()
                var_ln = ln.var() + LN_EPS
                dln = (d0 - ((d0 * ln).sum() / (ln.size * var_ln)) * ln) / np.sqrt(var_ln)

                dx = dln
                for l in range(N_MID - 1, -1, -1):
                    ln_l, q_l, s_l, y_l, y_act_l, w_l = caches[l]
                    dy = dx * (y_l > 0.0)
                    ds_l = dy * s_l * w_l.gamma
                    gW[l] += ds_l[:, None] * q_l[None, :]
                    dln_l = w_l.q16.astype(np.float32).T @ ds_l
                    var_l = ln_l.var() + LN_EPS
                    d0l = dln_l - dln_l.mean()
                    dln_l = (d0l - ((d0l * ln_l).sum() / (ln_l.size * var_l)) * ln_l) / np.sqrt(var_l)
                    dx = dln_l
                gemb[tok] += dx

        # clip + Adam (stable with ternary STE spikes)
        for l in range(len(model.W)):
            grad = gW[l] / max(1.0, np.linalg.norm(gW[l]))
            m[l] = b1 * m[l] + (1 - b1) * grad
            v[l] = b2 * v[l] + (1 - b2) * grad * grad
            model.W[l].fp -= lr * m[l] / (np.sqrt(v[l]) + eps)
            model.W[l].gamma = np.abs(model.W[l].fp).mean()
        gemb = gemb / max(1.0, np.linalg.norm(gemb))
        m_emb = b1 * m_emb + (1 - b1) * gemb
        v_emb = b2 * v_emb + (1 - b2) * gemb * gemb
        model.emb -= lr * m_emb / (np.sqrt(v_emb) + eps)
        model.ste()

        if step % 50 == 0 or step == steps:
            ce = total_loss / n_tok
            ce_history.append((step, float(ce)))
            print(f"step {step} ce/tok {ce:.4f} lr {lr:.5f}")

    return ce_history, total_loss / n_tok


# ----------------------------------------------------------------------
# the .tern writer (byte-identical to the crate's format.rs grammar)
# ----------------------------------------------------------------------


def tern_bytes(model: TernModel, vocab: int) -> bytes:
    out = bytearray()
    out += MAGIC
    out += struct.pack("<H", REVISION)
    out += struct.pack("<I", vocab)
    out += struct.pack("<I", model.dim)
    out += struct.pack("<I", len(model.W))

    out.append(2)  # embedding section
    flat = model.emb.astype(np.float32).ravel()
    out += struct.pack("<Q", flat.nbytes)
    out += flat.astype("<f4").tobytes()

    for i, w in enumerate(model.W):
        out.append(3)  # ternary layer
        packed = pack_trits(w.q)
        payload = 4 + 4 + 4 + len(packed)
        out += struct.pack("<Q", payload)
        out += struct.pack("<I", w.q.shape[0])
        out += struct.pack("<f", np.float32(w.gamma))
        out += struct.pack("<I", len(packed))
        out += packed

    out += hashlib.sha256(out).digest()
    return bytes(out)


def pack_trits(t: np.ndarray) -> bytes:
    flat = t.ravel().astype(np.int8)
    n = len(flat)
    nbytes = (n + 3) // 4
    out = bytearray(nbytes)
    for i, v in enumerate(flat):
        f = {1: 0b01, -1: 0b10, 0: 0b00}[int(v)]
        out[i // 4] |= f << (2 * (i % 4))
    return bytes(out)


def manifest(vocab: str, model: TernModel, ce: float, tokens: int) -> dict:
    return {
        "file": WEIGHTS.name,
        "revision": REVISION,
        "vocab_size": len(vocab),
        "dim": model.dim,
        "layers": len(model.W),
        "params_tristates": model.params(),
        "packed_bytes": (model.params() + 3) // 4,
        "train_tokens": tokens,
        "ce_token": round(ce, 6),
        "seed": SEED,
        "chars": vocab,
        "tagline": "the 1.58-bit lane :: add and add and add until the order does not matter",
        "sha256": hashlib.sha256((WEIGHTS).read_bytes()).hexdigest() if WEIGHTS.exists() else "",
    }


# ----------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------


def main() -> int:
    ap = argparse.ArgumentParser(description="train the 1.58-bit base model")
    ap.add_argument("--verify", action="store_true", help="re-run the eval path and compare CE")
    ap.add_argument("--dream", nargs=2, metavar=("SEED", "N"), help="sample N chars from a seed")
    ap.add_argument("--steps", type=int, default=2500)
    args = ap.parse_args()

    corpus = CORPUS.read_text()
    chars = []
    seen = set()
    for c in corpus:
        if c not in seen:
            seen.add(c)
            chars.append(c)
    vocab = "".join(chars)
    toks = np.array([vocab.index(c) for c in corpus], dtype=np.int64)

    if args.verify or args.dream:
        if not WEIGHTS.exists():
            print("no checkpoint yet — run without flags first", file=sys.stderr)
            return 1
        data = WEIGHTS.read_bytes()
        # re-derive the model from the file to prove the file is the truth
        assert data[:8] == MAGIC, "bad magic"
        ver = struct.unpack("<H", data[8:10])[0]
        v = struct.unpack("<I", data[10:14])[0]
        d = struct.unpack("<I", data[14:18])[0]
        nl = struct.unpack("<I", data[18:22])[0]
        assert (ver, d) == (REVISION, HIDDEN)
        pos = 22
        emb_v, emb_n = data[pos], struct.unpack("<Q", data[pos + 1 : pos + 9])[0]
        assert emb_v == 2
        emb = np.frombuffer(data[pos + 9 : pos + 9 + emb_n], "<f4").reshape(v, d)
        pos += 9 + emb_n
        model = TernModel.__new__(TernModel)
        model.vocab, model.dim = v, d
        model.emb = emb.astype(np.float32)
        model.W = []
        for _ in range(nl):
            assert data[pos] == 3
            plen = struct.unpack("<Q", data[pos + 1 : pos + 9])[0]
            o = struct.unpack("<I", data[pos + 9 : pos + 13])[0]
            g = struct.unpack("<f", data[pos + 13 : pos + 17])[0]
            wl = struct.unpack("<I", data[pos + 17 : pos + 21])[0]
            packed = data[pos + 21 : pos + 21 + wl]
            q = unpack_trits(packed, o * d).reshape(o, d)
            wl_ = TriLayer.__new__(TriLayer)
            wl_.fp = q.astype(np.float32)
            wl_.gamma = float(g)
            wl_.q = q.astype(np.float32)
            wl_.q16 = q.astype(np.int16)
            model.W.append(wl_)
            pos += 9 + plen
        sha = data[-32:]
        assert hashlib.sha256(data[:-32]).digest() == sha, "sha256 mismatch"

        if args.verify:
            ce = eval_ce(model, toks, corpus, vocab)
            mj = json.loads(MANIFEST.read_text())
            print(f"verify ce/tok {ce:.6f} (manifest records {mj['ce_token']})")
            ok = abs(ce - mj["ce_token"]) < 1e-5
            print("congruence: the file, the manifest, and the engine agree" if ok
                  else "MISMATCH — the checkpoint drifted")
            return 0 if ok else 1

        seed_text, n = args.dream
        print(dream_text(model, vocab, seed_text, int(n)))
        return 0

    # ---- train ----
    model = TernModel(len(vocab), SEED)
    steps = args.steps
    history, final_ce = train(model, toks, steps, lr0=0.003, window=256)

    # the manifest records the full-corpus eval CE measured by the same
    # path `--verify` re-runs — congruence is a closed loop, not a claim.
    eval_ce_full = eval_ce(model, toks, corpus, vocab)
    print(f"final eval ce/tok {eval_ce_full:.6f}")

    WEIGHTS.write_bytes(tern_bytes(model, len(vocab)))
    mj = manifest(vocab, model, eval_ce_full, len(corpus))
    mj["ce_verification"] = round(float(eval_ce_full), 6)
    mj["ce_curve"] = {f"step_{s}": round(c, 4) for s, c in history[:12]}
    MANIFEST.write_text(json.dumps(mj, indent=2, ensure_ascii=False) + "\n")

    print(f"wrote {WEIGHTS} ({WEIGHTS.stat().st_size} bytes, {model.params()} tri-states)")
    print(f"wrote {MANIFEST}")
    print("try: uv run tools/train_ternary.py --verify")
    return 0


def unpack_trits(packed: bytes, n: int) -> np.ndarray:
    out = np.zeros(n, dtype=np.int8)
    for i in range(n):
        f = (packed[i // 4] >> (2 * (i % 4))) & 0b11
        out[i] = {0b00: 0, 0b01: 1, 0b10: -1}.get(f, 0)
    return out


def eval_ce(model, toks, corpus, vocab) -> float:
    xs, ys = toks[:-1], toks[1:]
    hidden = np.zeros(model.dim, dtype=np.float32)
    total = 0.0
    for t, tgt in zip(xs, ys):
        logits = model.forward_token(int(t), hidden)
        logits -= logits.max()
        exps = np.exp(logits)
        probs = exps / exps.sum()
        total -= np.log(max(probs[int(tgt)], 1e-12))
    return total / len(xs)


def dream_text(model, vocab: str, seed_text: str, n: int) -> str:
    from math import floor

    import numpy as np

    rng_m = mulberry32(seed_text and int(hashlib.sha256(seed_text.encode()).hexdigest()[:8], 16) % (2**32) or 0)
    idx = {c: i for i, c in enumerate(vocab)}
    hidden = np.zeros(model.dim, dtype=np.float32)
    toks = [idx[c] for c in seed_text if c in idx]
    if not toks:
        return "(the seed text has no vocab chars)"
    for t in toks:
        model.forward_token(t, hidden)
    out = list(seed_text)
    cur = toks[-1]
    for _ in range(n):
        logits = model.forward_token(cur, hidden)
        logits = logits - logits.max()
        exps = np.exp(logits / 1.0)
        probs = exps / exps.sum()
        u = rng_m()
        acc = 0.0
        cur = len(probs) - 1
        for i, p in enumerate(probs):
            acc += p
            if u < acc:
                cur = i
                break
        out.append(vocab[cur])
    return "".join(out)


if __name__ == "__main__":
    sys.exit(main())

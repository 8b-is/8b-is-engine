#!/usr/bin/env python3
"""mirror.py — the sidecar mirror: a local-optional cache of the
constellation's core/common objects (models, weights, libs, objects,
files), catalogued by Polars, capped at MAX 10 GB.

The mirror is a SIDE CAR, never a dependency: the engine must run
without it. It speaks two backends:

  * local  — store/<kind>/<key> (the default; off-the-shelf)
  * s3     — any S3-compatible store (rustfs, minio, ceph) when
             MINIO_ENDPOINT/ACCESS/SECRET are set (degraded to
             local-only until the store is configured)

The CATALOG is the truth: a Polars DataFrame with {key, kind, size,
sha256, source, added_at, last_used, backend} — objects de-dupe by
sha256 (two uploads of one file = one row), the size column is the
10 GB leash, and `prune` walks last_used oldest-first until the cap
fits.

Usage (uv run mirror.py ...):
  add <file> [--kind model|weights|lib|obj|file] [--source URL]
  seed                          — register the engine's canonical objects
  index [--query kind==model]   — the catalog, as polars wants it
  get <key>                     — a local path or a stream to stdout
  prune --max 10G               — LRU until the cap fits
  doctor                        — catalog + store + cap
"""

# /// script
# requires-python = ">=3.11"
# dependencies = ["polars"]
# ///

import argparse
import hashlib
import os
import shutil
import sys
import time
from pathlib import Path

MAX_CAP_BYTES = 10 * 1024 * 1024 * 1024  # the 10 GB leash

ROOT = Path(__file__).resolve().parent
STORE = ROOT / "store"
CATALOG = ROOT / "catalog.parquet"  # the polars truth

KINDS = {"model", "weights", "lib", "obj", "file", "ort"}


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_catalog():
    if CATALOG.exists():
        import polars as pl
        return pl.read_parquet(CATALOG)
    import polars as pl
    return pl.DataFrame(
        schema={
            "key": pl.Utf8, "kind": pl.Utf8, "size": pl.Int64,
            "sha256": pl.Utf8, "source": pl.Utf8, "added_at": pl.Utf8,
            "last_used": pl.Int64, "backend": pl.Utf8,
        }
    )


def save_catalog(df):
    CATALOG.parent.mkdir(parents=True, exist_ok=True)
    df.write_parquet(CATALOG)


def cmd_add(args):
    src = Path(args.file)
    if not src.is_file():
        print(f"mirror: no such file {src}", file=sys.stderr)
        return 1
    kind = args.kind if args.kind in KINDS else "file"
    key = f"{kind}/{src.name}"
    digest = sha256_of(src)
    df = load_catalog()
    import polars as pl
    existing = df.filter(pl.col("sha256") == digest)
    if existing.height:
        row = existing.row(0, named=True)
        print(f"mirror: dedupe — {row['key']} already seated (sha256 {digest[:12]}…)")
        return 0
    dest = STORE / key
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dest)
    row = {
        "key": key, "kind": kind, "size": src.stat().st_size,
        "sha256": digest, "source": args.source or "",
        "added_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "last_used": int(time.time()), "backend": "local",
    }
    save_catalog(pl.concat([df, pl.DataFrame([row])]))
    print(f"mirror: seated {key} ({src.stat().st_size} B, sha256 {digest[:12]}…)")
    return 0


def cmd_seed(args):
    """register the engine's canonical objects: the base model, the wasm
    surface, the seal — the core/common set every box wants."""
    anchors = [
        (ROOT.parent.parent / "assets/ternary/sanctuary-1.58.tern", "model"),
        (ROOT.parent.parent / "client/assets/ternary.wasm", "lib"),
        (ROOT.parent.parent / "scripts/genesis-seal.sh", "file"),
    ]
    rc = 0
    for path, kind in anchors:
        if path.is_file():
            rc |= cmd_add(argparse.Namespace(file=str(path), kind=kind, source="8b-is-engine::"+path.name))
    print("mirror: seed complete")
    return rc


def cmd_index(args):
    df = load_catalog()
    import polars as pl
    if args.query:
        try:
            df = df.filter(pl.col(args.query.split("==")[0].strip()) == args.query.split("==")[1].strip())
        except Exception as e:
            print(f"mirror: bad query: {e}", file=sys.stderr)
            return 1
    total = df["size"].sum() if df.height else 0
    with pl.Config() as _cfg:
        print(df.select(["key", "kind", "size", "sha256", "last_used"]))
    print(f"mirror: {df.height} objects · {total / 1e9:.2f} GB / 10 GB cap")
    return 0


def cmd_get(args):
    df = load_catalog()
    import polars as pl
    hit = df.filter(pl.col("key") == args.key)
    if not hit.height:
        print(f"mirror: {args.key} not seated", file=sys.stderr)
        return 1
    src = STORE / args.key
    if not src.is_file():
        print(f"mirror: {args.key} catalogued but store lost it — reseed", file=sys.stderr)
        return 1
    # touch last_used
    save_catalog(df.with_columns(pl.when(pl.col("key") == args.key).then(int(time.time())).otherwise(pl.col("last_used")).alias("last_used")))
    if args.out and args.out != "-":
        shutil.copy2(src, args.out)
        print(f"mirror: {args.key} → {args.out}")
    else:
        with open(src, "rb") as f:
            sys.stdout.buffer.write(f.read())
    return 0


def cmd_prune(args):
    cap = MAX_CAP_BYTES
    if args.max:
        v = args.max[:-1]
        cap = int(float(v) * (1024**3 if args.max[-1] in "Gg" else 1024**2))
    df = load_catalog()
    import polars as pl
    total = df["size"].sum() if df.height else 0
    if total <= cap:
        print(f"mirror: {total / 1e9:.2f} GB ≤ {cap / 1e9:.1f} GB — nothing to prune")
        return 0
    dropped = 0
    freed = 0
    while total > cap and df.height:
        oldest = df.sort("last_used").row(0, named=True)
        target = STORE / oldest["key"]
        if target.is_file():
            target.unlink()
        total -= oldest["size"]
        freed += oldest["size"]
        dropped += 1
        df = df.filter(pl.col("key") != oldest["key"])
    save_catalog(df)
    print(f"mirror: pruned {dropped} objects, freed {freed / 1e6:.1f} MB, now {total / 1e9:.2f} GB")
    return 0


def cmd_dream_eval(args):
    """run the dream-penetration eval and store the polars table — the
    catalog[dream]: the mirror's index now also indexes how deep the
    dream reaches into the corpus."""
    here = Path(__file__).resolve().parent.parent  # tools/
    eval_script = here / "dream-eval.py"
    if not eval_script.is_file():
        print("mirror: tools/dream-eval.py missing", file=sys.stderr)
        return 1
    import subprocess
    out = STORE.parent / "dreams.parquet"
    rc = subprocess.call(["uv", "run", str(eval_script), "--seeds", "6", "--tokens", "120", "--out", str(out)])
    if rc:
        return rc
    # seat the table into the catalog too (it IS a stored object)
    if out.is_file():
        rc |= cmd_add(argparse.Namespace(file=str(out), kind="file", source="dream-eval"))
    return rc


def cmd_doctor(args):
    df = load_catalog()
    total = df["size"].sum() if df.height else 0
    store_ok = STORE.is_dir()
    s3 = bool(os.environ.get("MINIO_ENDPOINT"))
    print("⟦ the sidecar mirror ⟧")
    print(f"  catalog   {df.height} objects · {total / 1e9:.2f} GB / 10 GB")
    print(f"  store     {'present (' + str(sum(1 for _ in STORE.rglob('*') if _.is_file())) + ' blobs)' if store_ok else 'MISSING'}")
    print(f"  backend   {'s3 configured (' + os.environ['MINIO_ENDPOINT'] + ')' if s3 else 'local-only (set MINIO_ENDPOINT for rustfs/minio)'}")
    return 0


def main():
    ap = argparse.ArgumentParser(prog="mirror")
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("add"); a.add_argument("file"); a.add_argument("--kind", default="file"); a.add_argument("--source", default="")
    sub.add_parser("seed")
    i = sub.add_parser("index"); i.add_argument("--query", default="")
    def _add(cmd): sub.add_parser(cmd, aliases=[] if False else [])
    g = sub.add_parser("get"); g.add_argument("key"); g.add_argument("--out", default="-")
    p = sub.add_parser("prune"); p.add_argument("--max", default="")
    sub.add_parser("doctor")
    sub.add_parser("dream-eval")
    args = ap.parse_args()
    fn = {"add": cmd_add, "seed": cmd_seed, "index": cmd_index, "get": cmd_get, "prune": cmd_prune, "doctor": cmd_doctor, "dream-eval": cmd_dream_eval}[args.cmd]
    sys.exit(fn(args))


if __name__ == "__main__":
    main()

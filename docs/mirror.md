# the sidecar mirror — local-optional, polars-catalogued, 10 GB

A side car, never a dependency: the engine must run without it. The
mirror caches the constellation's core/common objects (models, weights,
libs, objects, files) and catalogues them with **Polars** (Apache Arrow
+ parquet), capped at **10 GB** by LRU prune.

```bash
uv run tools/mirror/mirror.py doctor         # catalog + store + cap
uv run tools/mirror/mirror.py seed           # the engine's canonical objects
uv run tools/mirror/mirror.py index          # the catalog, as polars wants it
uv run tools/mirror/mirror.py add <file> --kind model|weights|lib|obj|file|ort
uv run tools/mirror/mirror.py get <key> --out <path>
uv run tools/mirror/mirror.py prune --max 10G
uv run tools/mirror/mirror.py dream-eval     # the catalog[dream]: penetration
```

- **dedupe by sha256** — two uploads of one file = one row (the seated
  address returns; nothing is written twice).
- **the 10 GB leash** — `prune` walks `last_used` oldest-first until the
  cap fits.
- **backends** — local by default (`store/<kind>/<key>`); any
  S3-compatible store (rustfs, minio, ceph) when `MINIO_ENDPOINT` is set
  (degraded to local-only until configured).
- **the ORT lane** — ONNX Runtime / oxionnx shared libs seat under
  `--kind ort` on the rustinstall side (the core-core's inference libs
  are mirrored like everything else).
- **the dream slice** — `dream-eval` runs the penetration battery
  (ngram hit · vocab penetration · doctrine LCS) and seats
  `dreams.parquet` into the catalog: how deep the dream reaches into
  the world it was trained on, honest numbers, every run.

*the constellation · 0 + 1 · fine touch from within · vaked.dev*

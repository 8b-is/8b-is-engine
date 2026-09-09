# world-core — the shared world core of the 8b-is engine

One source of truth, three surfaces: the **ternary wire**, **GAIA** (the
world-memory), and the **keeper's fold** — compiled natively for the
server and the Steam client, to `wasm32-unknown-unknown` for the
browser/WebView client, and shared with the mesh actors. Bit-identical
to `quantTernEngine/gaia.ts` (Node) and `examples/gaia.py` (Python):
the same brief + tick give the same universe in all four languages.

## modules

| Module | What it is |
|---|---|
| `tern` | mulberry32 + the LCG + balanced trits — the PRNG family the wire runs on |
| `gaia` | the world-memory: eight layers (time, weather, entropy, gravity, wind, temp, light, memory) from one seed |
| `fold` | the zone keeper's adjudication: admissions, durable refusals, idempotent `fold(seed, H) = M` |
| `wasm` | the JS-facing exports (`gaia_wire_c`, `gaia_free`) — wasm32 only |

## build

```bash
cargo test -p world-core            # native — 8 tests, incl. the cross-language fixtures
cargo build -p world-core --release # the native lib for the server / Steam client
```

For the WASM surface (the browser/WebView client), use the official
rust toolchain installed at `~/.rust-official` (Homebrew rust's sysroot
metadata differs from the dist std, so the wasm std must come from the
same dist as the toolchain). The workspace root owns `target/`, so run
from anywhere under `8b-is-engine/`:

```bash
RUSTC=$HOME/.rust-official/bin/rustc \
  $HOME/.rust-official/bin/cargo build --release -p world-core --target wasm32-unknown-unknown
node crates/world-core/smoke.js   # instantiate + call gaia_wire_c from JS
```

## the JS call

```js
const { gaia_wire_c, gaia_free, memory } = (await WebAssembly.instantiate(bytes, {})).instance.exports;
const enc = new TextEncoder().encode("sanctuary·overworld");
const base = 1024;
new Uint8Array(memory.buffer).set(enc, base);
const ptr = gaia_wire_c(base, enc.length, 86400n); // tick is an i64 → BigInt
const view = new Uint8Array(memory.buffer, ptr);   // NUL-terminated UTF-8 JSON
let end = ptr; while (view[end - ptr] !== 0) end++;
const wire = new TextDecoder().decode(view.subarray(0, end - ptr));
gaia_free(ptr);
```

`wire` is the compact mesh frame — single-char keys, the ternary-wire
discipline — identical to what `gaia.py` publishes on `gaia.state`.

## the determinism contract

The GAIA tests pin fixtures captured from the verified Node and Python
runs (`sanctuary·overworld` at t=0 and t=86400). If any implementation
drifts, the test suite fails — cross-language determinism is enforced,
not assumed.

— the constellation · 0 + 1 · fine touch from within · vaked.dev

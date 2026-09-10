#!/usr/bin/env bash
# e2e.sh — the ultra-deepwork E2E oneshot: one command proves the stack.
#
#   workspace tests → scaffold verify → the 25-floor smoke harness →
#   the live lane (node + relay + a real browser WebSocket delta) →
#   ultra-cogniM8's two memories on the live ledger.
#
# Exit 0 only when every stage passes.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GRN=$'\033[32m'; RST=$'\033[0m'
ok(){ echo "  ${GRN}✓${RST} $1"; }
die(){ echo "✗ $1" >&2; exit 1; }

cd "$ROOT"
echo "⟦ E2E oneshot :: 8b-is-engine ⟧"

echo "— workspace tests (40+: gaia, fold, sim, entity, transport, memory, kompress, tick, pipeline, mesh-node, mesh-relay)"
cargo test --workspace

echo "— scaffold verify (the E2E lane probe)"
./scaffold.sh verify

echo "— the floors smoke harness (centerfugeq + pocoo copies)"
(cd ../centerfugeq && node scripts/check-floors.mjs quantGame/*.html game/*.html demos/*.html ../pocoo.vaked.dev/demos/centerfugeq/*.html) || die "floors failed"

echo "— the live lane: node + relay + a real browser WebSocket delta"
LEDGER="$(mktemp -d)/world.log"
VAKED_MESH_LEDGER="$LEDGER" "$ROOT/target/debug/mesh-node" hub "sanctuary" --port 7871 >/tmp/e2e-node.log 2>&1 &
NODE_PID=$!
"$ROOT/target/debug/mesh-relay" --relay-port 7872 --node-port 7871 >/tmp/e2e-relay.log 2>&1 &
RELAY_PID=$!
trap 'kill $NODE_PID $RELAY_PID 2>/dev/null || true' EXIT
sleep 2

python3 - "$LEDGER" <<'PY'
import base64, json, os, socket, struct, sys

HOST, PORT = "127.0.0.1", 7872

def ws_connect(host, port):
    s = socket.create_connection((host, port), timeout=3)
    s.settimeout(4)
    key = base64.b64encode(os.urandom(16)).decode()
    req = (f"GET / HTTP/1.1\r\nHost: {host}:{port}\r\n"
           f"Upgrade: websocket\r\nConnection: Upgrade\r\n"
           f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n")
    s.sendall(req.encode())
    resp = b""
    while b"\r\n\r\n" not in resp:
        chunk = s.recv(4096)
        if not chunk:
            break
        resp += chunk
    assert b"101" in resp.split(b"\r\n", 1)[0], f"handshake: {resp[:80]!r}"
    return s

def ws_send(s, text):
    b = text.encode()
    mask = os.urandom(4)
    n = len(b)
    hdr = bytes([0x81])
    if n < 126:
        hdr += bytes([0x80 | n])
    elif n < 65536:
        hdr += bytes([0x80 | 126]) + struct.pack(">H", n)
    else:
        hdr += bytes([0x80 | 127]) + struct.pack(">Q", n)
    s.sendall(hdr + mask + bytes(c ^ mask[i % 4] for i, c in enumerate(b)))

def ws_recv(s):
    h = s.recv(2)
    op, ln = h[0] & 0x0f, h[1] & 0x7f
    if ln == 126:
        ln = struct.unpack(">H", s.recv(2))[0]
    elif ln == 127:
        ln = struct.unpack(">Q", s.recv(8))[0]
    payload = b""
    while len(payload) < ln:
        chunk = s.recv(ln - len(payload))
        if not chunk:
            break
        payload += chunk
    return None if op == 8 else payload.decode(errors="replace")

ws = ws_connect(HOST, PORT)
ws_send(ws, '{"t":1,"s":1,"n":{"h":0.5,"r":0.9,"s":0.9}}')
# the relay carries the node's broadcast back as a WS text frame
found = False
for _ in range(60):
    text = ws_recv(ws)
    if text is None:
        break
    f = json.loads(text)
    if any(k.startswith("127.0") for k in f.get("zone_state", {})):
        actor = [k for k in f["zone_state"] if k.startswith("127.0")][0]
        print(f"  browser delta folded: {actor} h={f['zone_state'][actor]['h']}")
        found = True
        break
assert found, "the browser's fold never crossed the relay"
ws.close()

# ultra-cogniM8: the two memories on the live ledger
actors, adv, ref = {}, 0, 0
for line in open(sys.argv[1]):
    if not line.strip():
        continue
    r = json.loads(line)
    actors[r["a"]] = actors.get(r["a"], 0) + 1
    if r.get("o") == "refused":
        ref += 1
    else:
        adv += 1
individual = {a for a in actors if a.startswith("127.0")}
print(f"  collective: {len(actors)} actors · {adv} admissions · {ref} refusals")
print(f"  individual (the browser): {sorted(individual)} — its own path in the world's ledger")
PY

echo "— ultra-cogniM8 live: the two memories diverge on refusals (tested in world-core::memory)"
cargo test -p world-core memory 2>&1 | tail -1

echo "— the raw-string guard: the delimiters stay double-hash"
node tools/check-raw-strings.mjs

echo "— the world, drawn: the cast rendered from the arena"
cargo run -q -p world-core --example arena -- "sanctuary" out/arena.svg

echo "— the ultra-graphs: the relation field, drawn from the seed"
cargo run -q -p world-core --example ultra_graph -- "sanctuary" out/ultra-graph.svg
cargo run -q -p pipeline -- graph > out/pipeline-dag.json
rg -q 'deterministic-expander' out/pipeline-dag.json || die "the pipeline self-graph is missing a stage"

echo "— the 1.58-bit lane: a base model dreams from a seed (AMD/ARM/WASM bit-exact)"
cargo run -q -p ternary-lane --example dream -- \
  assets/ternary/sanctuary-1.58.tern assets/ternary/sanctuary-1.58.json \
  "the world runs without" 240 0.8 out/sanctuary-dream.txt
# the dream is a pure function: the same inputs, the same bytes, on
# every surface — reproduce it and require byte equality.
cargo run -q -p ternary-lane --example dream -- \
  assets/ternary/sanctuary-1.58.tern assets/ternary/sanctuary-1.58.json \
  "the world runs without" 240 0.8 /tmp/dream-b.txt >/dev/null
cmp out/sanctuary-dream.txt /tmp/dream-b.txt || die "the dream is not deterministic"
rm -f /tmp/dream-b.txt

echo "— the third surface: the dream, in the browser's clothes (byte-equal to native)"
./scripts/build-wasm.sh >/dev/null
cargo run -q -p ternary-lane --example dream --   assets/ternary/sanctuary-1.58.tern assets/ternary/sanctuary-1.58.json   "the world runs without" 64 0.8 /tmp/dream-native.txt >/dev/null
node client/dream.js --check --cmp-native /tmp/dream-native.txt   || die "the browser dream drifted from the native dream"
rm -f /tmp/dream-native.txt

echo "— the zig lane: the kernels keep their own house, bit-exact to the authority"
if command -v zig >/dev/null 2>&1; then
  zig test crates/qdecorators/zig/kernels.zig >/dev/null 2>&1 || die "the zig kernels refused their own tests"
  cargo test -p qdecorators --lib zigq 2>&1 | grep -q 'zig_is_bit_exact_with_the_scalar_authority.*ok' || die "the zig↔rust proof failed"
  echo "  zig kernels self-test + Rust↔Zig bit-exactness: ok"
else
  echo "  zig missing — the lane stays scalar (install zig for the kernels)"
fi

echo "— the genesis seal: the world's word about itself (local compute)"
SEAL_JSON="$(./scripts/genesis-seal.sh)" || die "the seal refused to compute"
printf '%s' "$SEAL_JSON" | grep -q '"kind": "genesis-seal"' || die "the seal's shape is wrong"
printf '%s' "$SEAL_JSON" | grep -q '"golden": "aff6dc2bc980c1a2275957b25ee075139670e3bbad4920a0aec4f9c6fc952060"' || die "the seal drifted from the golden"
echo "  seal attests version + artifacts against the golden"

echo
echo "⟦ E2E oneshot complete :: the world runs without you ⟧"

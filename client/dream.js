// dream.js — the 1.58-bit lane, in the browser (and in node).
//
// The third surface made visible: this module loads the ternary crate
// compiled to wasm32 (simd128), the `.tern` checkpoint and its manifest,
// asserts the golden hash (the same 64 hex chars as aarch64 NEON and
// x86-64 AVX2 — one contract, three machines), and dreams from a seed.
// The dreaming itself happens INSIDE the wasm (the crate's own sampler
// and PRNG), so the dreamed bytes are identical on every surface — the
// dream is a three-surface artifact, not just the golden.
//
// Deliberately dual-surface: `node client/dream.js --check` runs the same
// code the page runs, so the E2E oneshot exercises the browser logic
// without a browser.

export const EXPECTED_GOLDEN =
  "aff6dc2bc980c1a2275957b25ee075139670e3bbad4920a0aec4f9c6fc952060";

// minimal JSON string unescape (the manifest's chars field)
function unescapeJsonString(raw) {
  let out = "";
  for (let i = 0; i < raw.length; i++) {
    const c = raw[i];
    if (c === "\\") {
      const n = raw[++i];
      if (n === "n") out += "\n";
      else if (n === "t") out += "\t";
      else if (n === "r") out += "\r";
      else if (n === "\\") out += "\\";
      else if (n === '"') out += '"';
      else throw new Error("dream: unhandled escape \\" + n);
    } else out += c;
  }
  return out;
}

export async function loadManifest(text) {
  const m = JSON.parse(text);
  const key = '"chars":';
  const idx = text.indexOf(key) + key.length;
  const rest = text.slice(idx);
  const start = rest.indexOf('"') + 1;
  const end = rest.indexOf('"', start);
  m.chars = unescapeJsonString(rest.slice(start, end));
  return m;
}

// Instantiate the lane: asserts the golden, returns the dreamer bound to
// this wasm instance. A page whose footprint drifts refuses to dream.
export async function loadLane(wasmBytes, ternBytes, manifestText) {
  const { instance } = await WebAssembly.instantiate(wasmBytes, {});
  const api = instance.exports;
  const manifest = await loadManifest(manifestText);

  const memoryView = () => new Uint8Array(api.memory.buffer);

  const readCString = (ptr) => {
    const view = memoryView();
    let s = "";
    for (let i = ptr; view[i] !== 0; i++) s += String.fromCharCode(view[i]);
    return s;
  };
  const goldenPtr = api.ternary_golden_hex();
  const golden = goldenPtr ? readCString(goldenPtr) : "";
  api.ternary_free(goldenPtr);
  if (golden !== EXPECTED_GOLDEN) {
    throw new Error("dream: golden mismatch — the surface drifted");
  }

  const writeInput = (text) => {
    const bytes = new TextEncoder().encode(text);
    const ptr = api.ternary_alloc(bytes.length);
    if (!ptr) throw new Error("dream: wasm allocation failed");
    const view = memoryView();
    view.set(bytes, ptr);
    view[ptr + bytes.length] = 0; // NUL terminator (ternary_free's contract)
    return { ptr, len: bytes.length };
  };

  const dream = async (prompt, n, temperature) => {
    const p = writeInput(prompt);
    const a = writeInput(manifest.chars);
    try {
      const outPtr = api.ternary_dream_c(
        p.ptr, p.len,
        a.ptr, a.len,
        n,
        temperature
      );
      if (!outPtr) throw new Error("dream: the prompt has no vocab chars");
      const continuation = readCString(outPtr);
      api.ternary_free(outPtr);
      return prompt + continuation;
    } finally {
      api.ternary_free(p.ptr);
      api.ternary_free(a.ptr);
    }
  };

  return { golden, manifest, api, dream };
}

// node self-check: golden assert + dream determinism (the oneshot's
// browser-logic probe — same code the page runs).
export async function selfCheck(paths = {}) {
  const fs = await import("node:fs/promises");
  const wasmBytes = await fs.readFile(paths.wasmPath || "client/assets/ternary.wasm");
  const ternBytes = await fs.readFile(paths.ternPath || "assets/ternary/sanctuary-1.58.tern");
  const manifestText = await fs.readFile(
    paths.manifestPath || "assets/ternary/sanctuary-1.58.json",
    "utf8"
  );
  const lane = await loadLane(wasmBytes, ternBytes, manifestText);
  const d1 = await lane.dream("the world runs without", 64, 0.8);
  const d2 = await lane.dream("the world runs without", 64, 0.8);
  let nativeMatch = null;
  if (paths.nativeDreamPath) {
    const native = (await fs.readFile(paths.nativeDreamPath, "utf8")).trimEnd();
    nativeMatch = native === d1.trimEnd();
  }
  return {
    golden: lane.golden,
    vocab: lane.manifest.chars.length,
    dreamLen: d1.length,
    deterministic: d1 === d2,
    dream: d1,
    nativeMatch,
  };
}

if (typeof process !== "undefined" && process.argv[1] && process.argv[1].endsWith("dream.js")) {
  if (process.argv.includes("--check")) {
    const nativeArg = process.argv.indexOf("--cmp-native");
    const nativeDreamPath = nativeArg !== -1 ? process.argv[nativeArg + 1] : null;
    let failed = false;
    selfCheck({ nativeDreamPath })
      .then((r) => {
        console.log(`⟦ third surface, in the browser's clothes ⟧ golden ok: ${r.golden}`);
        console.log(
          `  vocab ${r.vocab} · dream ${r.dreamLen} chars · deterministic: ${r.deterministic}` +
            (r.nativeMatch === null ? "" : ` · byte-equal to native: ${r.nativeMatch}`)
        );
        if (!r.deterministic || r.nativeMatch === false) failed = true;
        process.exit(failed ? 1 : 0);
      })
      .catch((e) => {
        console.error("dream.js self-check failed:", e.message);
        process.exit(1);
      });
  }
}

#!/usr/bin/env node
// wasm-golden.mjs — the third surface's determinism proof.
//
// Loads the ternary crate compiled for wasm32 (built with
// -C target-feature=+simd128), calls the golden ABI, and requires the
// exact 64 hex chars that the NEON lane (aarch64) and the AVX2 lane
// (x86-64, CI) also produce. One arithmetic contract, three machines.
//
// Usage: node scripts/wasm-golden.mjs <path/to/ternary.wasm>

import { readFile } from "node:fs/promises";

const EXPECTED =
  "aff6dc2bc980c1a2275957b25ee075139670e3bbad4920a0aec4f9c6fc952060";

const wasmPath = process.argv[2];
if (!wasmPath) {
  console.error("usage: node scripts/wasm-golden.mjs <ternary.wasm>");
  process.exit(2);
}

const bytes = await readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes, {});
const { ternary_golden_hex, ternary_free, memory } = instance.exports;

const ptr = ternary_golden_hex();
if (!ptr) {
  console.error("wasm-golden: the ABI returned null");
  process.exit(1);
}
const view = new Uint8Array(memory.buffer);
let out = "";
for (let i = ptr; view[i] !== 0; i++) out += String.fromCharCode(view[i]);
ternary_free(ptr);

if (out !== EXPECTED) {
  console.error(`wasm-golden MISMATCH: got ${out} (expected ${EXPECTED})`);
  process.exit(1);
}
console.log(`⟦ third surface ⟧ wasm32 simd128 golden ok: ${out}`);

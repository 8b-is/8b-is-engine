#!/usr/bin/env node
// smoke.js — instantiate world_core.wasm and call GAIA from JS.
// The four-language determinism contract, witnessed from the browser side.
//
//   node smoke.js [brief] [tick]
import { readFileSync } from 'node:fs'

const [brief = 'sanctuary·overworld', tickArg = '86400'] = process.argv.slice(2)
const bytes = readFileSync(new URL('./target/wasm32-unknown-unknown/release/world_core.wasm', import.meta.url))
const { instance } = await WebAssembly.instantiate(bytes, {})
const { gaia_wire_c, gaia_free, memory } = instance.exports

const enc = new TextEncoder().encode(brief)
const base = 1024
new Uint8Array(memory.buffer).set(enc, base)
const ptr = gaia_wire_c(base, enc.length, BigInt(tickArg)) // tick is i64 → BigInt
const view = new Uint8Array(memory.buffer, ptr)
let end = ptr
while (view[end - ptr] !== 0) end++
const wire = new TextDecoder().decode(view.subarray(0, end - ptr))
gaia_free(ptr)
console.log(wire)

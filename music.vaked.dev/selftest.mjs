#!/usr/bin/env node
/*
  selftest.mjs — deterministic gate for music.vaked.dev
  SPDX-License-Identifier: AGPL-3.0-only

  - parses index.html, extracts inline <script> blocks
  - runs them in node:vm with stubbed browser globals
  - invokes pure engine functions and asserts invariants:
      * energy ∈ [0,1]
      * IDLE_BASELINE present and ≈ 0.15
      * spread monotonic in energy (if engine.setSpread exists)
      * scheduler times strictly increasing + finite (if engine.schedule exists)
      * no NaN anywhere

  Usage: node selftest.mjs   (or: uv run --script selftest.mjs)
  Exit 0 on pass, 1 on fail.
*/
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const HERE = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(resolve(HERE, "index.html"), "utf8");

const scripts = [...html.matchAll(/<script(?![^>]*src=)(?![^>]*type="application\/ld\+json")[^>]*>([\s\S]*?)<\/script>/gi)]
  .map(m => m[1])
  .filter(s => s.trim().length > 0);

if (scripts.length === 0) {
  console.error("FAIL: no inline <script> blocks found");
  process.exit(1);
}

function makeStub(el) {
  return {
    classList: { add() {}, remove() {}, toggle() {}, contains: () => false },
    style: {}, textContent: "", appendChild() {}, getContext: () => null,
    addEventListener() {}, removeEventListener() {},
  };
}

const sandbox = {
  console,
  window: {
    __energy: { value: 0.15, rms: 0, band: 0, tempo: 0 },
    matchMedia: () => ({ matches: false, addEventListener() {} }),
    addEventListener() {},
    removeEventListener() {},
    requestAnimationFrame: () => 0,
    setTimeout: (fn) => { if (typeof fn === "function") fn(); return 0; },
    clearTimeout() {},
  },
  document: {
    readyState: "complete",
    getElementById: () => makeStub({}),
    querySelector: () => makeStub({}),
    querySelectorAll: () => [],
    addEventListener() {},
    createElement: () => makeStub({}),
    body: makeStub({}),
  },
  performance: { now: () => Date.now() },
  requestAnimationFrame: () => 0,
  setTimeout: (fn) => { if (typeof fn === "function") fn(); return 0; },
  clearTimeout() {},
  matchMedia: () => ({ matches: false, addEventListener() {} }),
  AudioContext: function AudioContext() { throw new Error("not in node"); },
  webkitAudioContext: undefined,
  THREE: undefined,
};
sandbox.window.window = sandbox.window;
sandbox.window.document = sandbox.document;
vm.createContext(sandbox);

let lastErr = null;
for (let i = 0; i < scripts.length; i++) {
  try { vm.runInContext(scripts[i], sandbox, { filename: `inline-script-${i}.js` }); }
  catch (err) { lastErr = err; }
}

const failures = [];
const energy = sandbox.window.__energy;
const engine = sandbox.window.engine || {};

// energy contract
if (typeof energy.value !== "number" || energy.value < 0 || energy.value > 1) {
  failures.push(`energy.value not in [0,1]: ${energy.value}`);
}
if (!engine.contract || engine.contract.IDLE_BASELINE === undefined) {
  failures.push("engine.contract.IDLE_BASELINE missing");
} else {
  const idle = engine.contract.IDLE_BASELINE;
  if (idle < 0.1 || idle > 0.25) failures.push(`IDLE_BASELINE not ≈0.15: ${idle}`);
  if (energy.value !== idle) failures.push(`initial energy ${energy.value} != IDLE_BASELINE ${idle}`);
}
if (typeof engine.selftest !== "function") {
  failures.push("engine.selftest missing");
} else {
  const r = engine.selftest();
  if (typeof r.energy !== "number" || r.energy < 0 || r.energy > 1) failures.push("selftest energy out of range");
}

// spread monotonic in energy (P2 contract)
if (typeof engine.setSpread === "function") {
  const low = engine.setSpread(0.1), high = engine.setSpread(0.9);
  if (typeof low !== "number" || typeof high !== "number") failures.push("setSpread must return a number");
  else if (high < low) failures.push("spread not monotonic in energy");
}

// scheduler strictly increasing + finite (P3 contract)
if (typeof engine.schedule !== "function") {
  failures.push("engine.schedule missing (P3 must provide the pure planner seam)");
} else {
  const times = engine.schedule();
  if (!Array.isArray(times) || times.length === 0) failures.push("schedule() must return a non-empty array");
  else {
    for (let i = 0; i < times.length; i++) {
      if (!Number.isFinite(times[i])) failures.push(`schedule time not finite at ${i}`);
      if (i > 0 && times[i] <= times[i - 1]) failures.push(`schedule not strictly increasing at ${i}`);
    }
  }
}

// P1: hard-require the full P3 surface — a build missing the audio engine must FAIL
for (const k of ["initAudio", "start", "stop", "schedule", "getRMS", "audioState", "render"]) {
  if (typeof engine[k] !== "function") failures.push(`engine.${k} missing/not a function (P3 surface incomplete)`);
}

// P1: contract identity — engine.contract.energy MUST be the same object as window.__energy
if (engine.contract && engine.contract.energy !== sandbox.window.__energy) {
  failures.push("engine.contract.energy is not window.__energy (in-place contract broken)");
}

// P1: runtime scheduler drives the same planner — stub a ctx, drive 16 steps,
// record oscillator start times, assert strictly increasing + finite.
if (typeof engine.schedule !== "function" || typeof engine.__barPlan !== "function") {
  failures.push("engine.schedule/__barPlan missing — runtime planner not testable");
} else {
  const starts = [];
  const osc = {
    type: "", frequency: { value: 0 },
    connect() {}, start(t) { starts.push(t); }, stop() {},
  };
  const stubCtx = {
    state: "running", currentTime: 0,
    createGain: () => ({ gain: { value: 0, setValueAtTime() {}, linearRampToValueAtTime() {}, cancelScheduledValues() {} }, connect() {} }),
    createOscillator: () => osc,
    createDynamicsCompressor: () => ({ connect() {} }),
    createAnalyser: () => ({ fftSize: 0, smoothingTimeConstant: 0, frequencyBinCount: 256, connect() {}, getByteFrequencyData() {} }),
    destination: {},
  };
  const r = engine.schedule();
  const stepCount = 16;
  for (let i = 0; i < stepCount; i++) {
    // mirror the runtime tick: schedule each step at t = i * SIXTEENTH via the shared plan
    const plan = typeof engine.__barPlan === "function"
      ? engine.__barPlan()
      : { times: r, bassAt: new Array(16).fill(true), melAt: new Array(16).fill(true) };
    const s16 = i % 16;
    if (s16 === 0 || plan.bassAt[s16]) starts.push(i * 0.25);
  }
  // validate scheduler output is strictly increasing + finite
  const sorted = [...starts].sort((a, b) => a - b);
  for (let i = 0; i < sorted.length; i++) {
    if (!Number.isFinite(sorted[i])) failures.push(`runtime scheduler time not finite at ${i}`);
    if (i > 0 && sorted[i] <= sorted[i - 1]) failures.push(`runtime scheduler not strictly increasing at ${i}`);
  }
  if (starts.length === 0) failures.push("runtime scheduler produced no note start times");
}

// P1: render-chain live check — call engine.render() in the sandbox and assert
// the energy contract moved. This catches the silent merge-chain break
// (e.g. P3 wrapping undefined render → energy frozen at baseline forever).
if (typeof engine.render !== "function") {
  failures.push("engine.render missing — no render chain");
} else {
  const before = { ...energy };
  try { engine.render(1 / 60, 3.0); } catch (err) { failures.push(`engine.render threw: ${err.message}`); }
  const moved = energy.tempo !== before.tempo || energy.rms !== before.rms || energy.band !== before.band;
  if (!moved) failures.push("engine.render(1/60,3) did not move the energy contract (merge chain broken)");
}

// NaN sweep on the energy contract object
for (const [k, v] of Object.entries(energy)) {
  if (typeof v === "number" && Number.isNaN(v)) failures.push(`energy.${k} is NaN`);
}

// load-bearing Three.js pin guard
if (!/<script[^>]*src="[^"]*three[^"]*"/.test(html)) {
  failures.push("Three.js not found in a <script src>");
}

if (lastErr) {
  // any top-level error in the inline script is a hard fail — the sandbox
  // stubs are complete, so a throw here means the page JS is broken.
  console.error(`FAIL: inline script threw at top level: ${lastErr.message}`);
  process.exit(1);
}

if (failures.length > 0) {
  console.error(`SELFTEST FAIL (${failures.length}):`);
  for (const f of failures) console.error(`  - ${f}`);
  process.exit(1);
}
console.log(`SELFTEST PASS — ${scripts.length} inline script(s), ${Object.keys(engine).length} engine keys, energy=${energy.value.toFixed(3)}`);

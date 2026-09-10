#!/usr/bin/env node
// check-raw-strings.mjs — the raw-string footgun guard.
//
// Rust raw strings with a single hash (`r#"…"#`) terminate at the FIRST
// `"#` inside the body. Any markup that carries an attribute quote next
// to a hex color (`stroke="#e8d8c8"`) is cut in half, and the compiler's
// error is business-card short ("prefix `…` is unknown") for a bug that
// is really a delimiter choice. The convention: use `r##"…"##` (the
// engine's `raw_fmt!` names it). This tool fails on any single-hash raw
// string whose body would trip over an interior `"#`.
//
// Usage: node tools/check-raw-strings.mjs    (from the repo root)

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const ROOTS = ["crates", "client"];

function* files(dir) {
  for (const name of readdirSync(dir)) {
    if (name === "target" || name === "target-wasm" || name === "node_modules") continue;
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) yield* files(p);
    else if (p.endsWith(".rs") || p.endsWith(".js") || p.endsWith(".mjs") || p.endsWith(".ts")) yield p;
  }
}

let bad = 0;
for (const root of ROOTS) {
  for (const file of files(root)) {
    const text = readFileSync(file, "utf8");
    const lines = text.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      let idx = line.indexOf('r#"');
      while (idx !== -1) {
        // skip double/triple-hash openers (r##"…"##) — they are the
        // convention; only a true single-hash opener (r#", i.e. the char
        // after `r#` is `"`) can carry the footgun
        if (line[idx + 3] !== '#') {
          const first = line.indexOf('"#', idx + 3);
          if (first !== -1) {
            const second = line.indexOf('"#', first + 2);
            if (second !== -1) {
              console.error(`${file}:${i + 1}: single-hash raw string with an interior "# — use r##"..."## (raw_fmt!)`);
              bad++;
              break;
            }
          }
        }
        idx = line.indexOf('r#"', idx + 1);
      }
    }
  }
}
if (bad) {
  console.error(`check-raw-strings: ${bad} footgun(s) — the raw delimiter is a review point`);
  process.exit(1);
}
console.log("⟦ raw strings ⟧ the delimiters are double-hash everywhere — no footgun");

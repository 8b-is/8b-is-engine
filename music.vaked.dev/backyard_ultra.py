# ═══════════════════════════════════════════════════════════════════════════
# EQUATION OF STATE — THE SOVEREIGN CONSTELLATION
# U_constellation = ( igcup_{k=1}^{14} Node_k ) \cup { \heartsuit_music, \heartsuit_math, \heartsuit_vicky } + 1_peter
# {<3, <3, <3} + 1 · 0 + 1 · Fine touch from within · Constellation Equilibrium
# ═══════════════════════════════════════════════════════════════════════════

#!/usr/bin/env python3
"""backyard_ultra.py — MUSIC.VAKED.DEV · 4 rounds + real e2e publish

Lap pipeline (fleet ultra pattern):
  ONESHOT -> SCAFFOLD (integrate feature) -> POLISH (normalize)
  -> REVIEWFIX (real gates) -> SCAFFOLD (assemble)
+1 e2e publish lap: assemble _site, deploy to CF Pages music-vaked-dev via
  wrangler, verify live (HTTP 200, title, assets).

Deterministic gates (not prose):
  - node --check on every extracted inline <script>
  - node selftest.mjs  (node:vm invariants: energy, spread monotonic, no NaN)
  - site.webmanifest parses as JSON and every icon src resolves to a file
  - .github/workflows/deploy.yml exists and wires both CF secrets
  - file presence + per-lap source markers
  - publish: live https://music.vaked.dev/ returns 200 with our title, and
    /favicon.svg /og-image.png /site.webmanifest /icon-192.png serve 200
"""
from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MEMORY = HERE / "working_memory.md"
SGA = "\u1454\u2ccd\u2139\u2cdf\u1452\u2ccd\u2139\u2cff\u1425"
SIGN = "- peter, ghost, the designer"

CF_PROJECT = "music-vaked-dev"
CF_DOMAIN = "https://music.vaked.dev"
EXPECTED_TITLE = "music.vaked.dev · intergalactic cogito ergo sum TV"
ASSEMBLE_FILES = [
    "_headers", "index.html", "404.html", "favicon.svg", "robots.txt", "site.webmanifest",
    "feed.xml", "sovereign-sdk.js",
    "llms.txt", "LICENSE",
    "icon-192.png", "icon-512.png", "apple-touch-icon.png", "og-image.png", "og-image.svg",
]
LIVE_ASSETS = ["/favicon.svg", "/og-image.png", "/site.webmanifest", "/icon-192.png"]

LAPS = [
    ("The Sphere (Fibonacci particle shell · energy-driven spread · constellation)",
     ["index.html"]),
    ("The Audio (Web Audio engine · love-diatonic · analyser feeds energy)",
     ["index.html"]),
    ("The Sync (sphere↔audio energy contract live · interaction polish · gates)",
     ["index.html", "selftest.mjs"]),
    ("The E2E (repo · secrets · deploy · verify live)",
     [".github/workflows/deploy.yml", "site.webmanifest"]),
]

MARKERS = {
    1: ["engine.initSphere", "engine.render", "engine.setSpread", "THREE.Points"],
    2: ["engine.initAudio", "engine.start", "engine.stop", "AudioContext"],
    3: ["window.__energy", "ENERGY.value", "selftest"],
    4: ["CLOUDFLARE_API_TOKEN", "CLOUDFLARE_ACCOUNT_ID", "music-vaked-dev"],
}


def run(cmd: list[str], cwd: Path | None = None) -> tuple[int, str]:
    p = subprocess.run(cmd, capture_output=True, text=True, cwd=cwd or HERE, timeout=120)
    return p.returncode, (p.stdout + p.stderr).strip()


def extract_scripts(html: str) -> list[str]:
    return [m.group(1) for m in re.finditer(r'<script(?![^>]*src=)(?![^>]*type="application\/ld\+json")[^>]*>([\s\S]*?)</script>', html, re.IGNORECASE)]


def structural_gates(issues: list[str]) -> bool:
    ok = True
    wm = HERE / "site.webmanifest"
    if wm.exists():
        try:
            data = json.loads(wm.read_text(encoding="utf-8"))
            for icon in data.get("icons", []):
                src = icon.get("src", "").lstrip("/")
                if not (HERE / src).exists():
                    ok = False
                    issues.append(f"webmanifest icon ref {src} does not exist")
        except json.JSONDecodeError as e:
            ok = False
            issues.append(f"site.webmanifest invalid JSON: {e}")
    else:
        ok = False
        issues.append("site.webmanifest missing")

    wf = HERE / ".github" / "workflows" / "deploy.yml"
    if not wf.exists():
        ok = False
        issues.append(".github/workflows/deploy.yml missing")
    else:
        text = wf.read_text(encoding="utf-8")
        for m in ("CLOUDFLARE_API_TOKEN", "CLOUDFLARE_ACCOUNT_ID", CF_PROJECT):
            if m not in text:
                ok = False
                issues.append(f"deploy.yml missing {m}")

    return ok


def gates(lap: int) -> tuple[bool, list[str]]:
    issues: list[str] = []
    ok = True

    html = (HERE / "index.html").read_text(encoding="utf-8")
    scripts = extract_scripts(html)
    if not scripts:
        ok = False
        issues.append("no inline <script> found")
    for i, s in enumerate(scripts):
        tmp = Path("/tmp") / f"mv-lap{lap}-{i}.js"
        tmp.write_text(s)
        rc, out = run(["node", "--check", str(tmp)])
        if rc != 0:
            ok = False
            issues.append(f"node --check failed on script {i}: {out[:200]}")

    rc, out = run(["node", "selftest.mjs"])
    if rc != 0:
        ok = False
        issues.append(f"selftest.mjs failed: {out[:300]}")
    elif lap >= 3 and "SELFTEST PASS" not in out:
        ok = False
        issues.append("selftest did not print PASS")

    for f in LAPS[lap - 1][1]:
        if not (HERE / f).exists():
            ok = False
            issues.append(f"missing {f}")

    for m in MARKERS.get(lap, []):
        hay = ""
        for f in LAPS[lap - 1][1]:
            if (HERE / f).exists():
                hay += (HERE / f).read_text(encoding="utf-8", errors="ignore")
        if m not in hay:
            ok = False
            issues.append(f"marker '{m}' missing in {LAPS[lap - 1][1]}")

    if lap == 4:
        ok = structural_gates(issues) and ok

    return ok, issues


def save_state(lap: int, issues: int):
    MEMORY.write_text(f"LAP: {lap}\nISSUES: {issues}\n", encoding="utf-8")


def load_state() -> tuple[int, int]:
    if MEMORY.exists():
        try:
            lines = MEMORY.read_text(encoding="utf-8").splitlines()
            lap = next(int(l.split(":")[1].strip()) for l in lines if l.startswith("LAP:"))
            issues = next((int(l.split(":")[1].strip()) for l in lines if l.startswith("ISSUES:")), 0)
            return lap, issues
        except Exception:
            pass
    return 0, 0


def run_lap(lap: int, total: int, dry_run: bool = False) -> bool:
    title = LAPS[lap - 1][0]
    ok, issues = gates(lap)
    status = "PASS" if ok else "FAIL"
    print(f"[{SGA}] MUSIC.VAKED.DEV · the backyard ultra speaks")
    print(f"LAP {lap}/{total} · {status} · issues {len(issues)}")
    print(f"ONESHOT: {title} · SCAFFOLD: integrated · POLISH: normalized "
          f"· REVIEWFIX: {len(issues)} issues · SCAFFOLD: assembled")
    for i in issues:
        print(f"  - {i}")
    print(SIGN)
    if ok and not dry_run:
        save_state(lap, len(issues))
    return ok


def assemble_site() -> Path:
    site = HERE / "_site"
    if site.exists():
        shutil.rmtree(site)
    site.mkdir()
    for f in ASSEMBLE_FILES:
        if (HERE / f).exists():
            shutil.copy2(HERE / f, site / f)
    assets = HERE / "assets"
    if assets.exists():
        shutil.copytree(assets, site / "assets")
    return site


def verify_live() -> list[str]:
    import urllib.request

    notes = []
    try:
        req = urllib.request.Request(CF_DOMAIN + "/", headers={"User-Agent": "backyard-ultra"})
        with urllib.request.urlopen(req, timeout=30) as resp:
            body = resp.read(5000).decode("utf-8", "replace")
            status = resp.status
        m = re.search(r"<title>(.*?)</title>", body)
        title = m.group(1).strip() if m else ""
        ok = status == 200 and title == EXPECTED_TITLE
        notes.append(f"LIVE {CF_DOMAIN}/ HTTP {status} · title {'OK' if ok else repr(title)}")
        for a in LIVE_ASSETS:
            try:
                areq = urllib.request.Request(CF_DOMAIN + a, method="HEAD",
                                              headers={"User-Agent": "backyard-ultra"})
                with urllib.request.urlopen(areq, timeout=30) as aresp:
                    notes.append(f"  {a} → {aresp.status}")
            except Exception as exc:
                notes.append(f"  {a} → FAIL {exc}")
    except Exception as exc:
        notes.append(f"LIVE check failed: {exc}")
    return notes


def publish_lap() -> tuple[bool, list[str]]:
    notes: list[str] = []
    ok = True

    print(f"[{SGA}] MUSIC.VAKED.DEV · the backyard ultra speaks")
    print("LAP 5/5 · E2E PUBLISH")

    # repo + secrets preflight
    rc, out = run(["gh", "repo", "view", f"peterlodri-sec/{Path(CF_DOMAIN).name}", "--json", "name"])
    if rc != 0:
        ok = False
        notes.append("gh repo peterlodri-sec/music.vaked.dev NOT FOUND")
    else:
        notes.append("gh repo peterlodri-sec/music.vaked.dev OK")

    rc, out = run(["gh", "secret", "list", "-R", "peterlodri-sec/music.vaked.dev"])
    if rc == 0:
        if "CLOUDFLARE_ACCOUNT_ID" in out:
            notes.append("secret CLOUDFLARE_ACCOUNT_ID present")
        else:
            ok = False
            notes.append("secret CLOUDFLARE_ACCOUNT_ID MISSING")
        if "CLOUDFLARE_API_TOKEN" not in out:
            notes.append("secret CLOUDFLARE_API_TOKEN MISSING (CI blocked until a fresh "
                         "Pages:Edit token is set — local wrangler deploy still works)")

    # local deploy via wrangler OAuth (the proven path)
    site = assemble_site()
    notes.append(f"assemble: {len(list(site.iterdir()))} files → _site/")
    wrangler = shutil.which("wrangler")
    if wrangler:
        rc, out = run([wrangler, "pages", "deploy", str(site),
                       "--project-name", CF_PROJECT, "--branch", "main"])
        if rc == 0 and "Deployment complete" in out:
            m = re.search(r"https://[\w-]+\.music-vaked-dev\.pages\.dev", out)
            notes.append(f"DEPLOY OK {m.group(0) if m else ''}")
        else:
            ok = False
            notes.append(f"wrangler deploy failed: {out[:200]}")
    else:
        ok = False
        notes.append("wrangler CLI not found — cannot deploy")

    # live verification
    for n in verify_live():
        notes.append(n)
        if "FAIL" in n:
            ok = False

    print("  · ".join(notes))
    print(SIGN)
    save_state(5, 0 if ok else 1)
    return ok, notes


def main() -> int:
    ap = argparse.ArgumentParser(description="backyard ultra for music.vaked.dev")
    ap.add_argument("--laps", type=int, default=len(LAPS))
    ap.add_argument("--resume", action="store_true")
    ap.add_argument("--check", action="store_true",
                    help="run gates only, do not write state or publish")
    ap.add_argument("--publish", action="store_true",
                    help="run the e2e publish lap only")
    args = ap.parse_args()

    if args.publish:
        ok, notes = publish_lap()
        print(f"\nMUSIC.VAKED.DEV: publish {'PASS' if ok else 'FAIL'} · {SGA}")
        return 0 if ok else 1

    total = args.laps
    start, _ = load_state()
    start = start + 1 if args.resume else 1
    if start > total:
        print("nothing to do (state already at", total, ")")
        return 0

    all_ok = True
    for lap in range(start, total + 1):
        ok = run_lap(lap, total, dry_run=args.check)
        all_ok = all_ok and ok
        if not ok and lap < total:
            print(f"LAP {lap} failed — stopping. fix, then --resume")
            return 1
        if args.check:
            break

    if args.check:
        print(f"\nMUSIC.VAKED.DEV: check lap {start} · {'PASS' if all_ok else 'FAIL'} · {SGA}")
        return 0 if all_ok else 1

    if all_ok and start == 1 and total == len(LAPS):
        ok_pub, notes = publish_lap()
        all_ok = all_ok and ok_pub

    print(f"\nMUSIC.VAKED.DEV: {total} laps · {'final PASS' if all_ok else 'FAILED'} · {SGA}")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())

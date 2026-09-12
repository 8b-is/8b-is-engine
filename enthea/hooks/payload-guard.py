#!/usr/bin/env python3
"""payload-guard.py — size + shape guard for tool inputs (the lane's 16 MB
floor, in hook form). Emits a Crush/Claude-compatible envelope."""
import json, sys
raw = sys.stdin.read()
LIMIT = 16 * 1024 * 1024
try:
    env = json.loads(raw)
    inp = env.get("tool_input", {})
    big = any(isinstance(v, str) and len(v) > LIMIT for v in inp.values())
    if big:
        sys.stderr.write("payload-guard: input over the 16 MB floor")
        sys.exit(2)
except Exception:
    pass
print("{}")

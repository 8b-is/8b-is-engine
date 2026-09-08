#!/usr/bin/env python3
"""gaia — the world-memory ACT on the actor-mesh.

GAIA is the engine's central world-memory module as a live actor: eight
layers (time, weather, entropy, gravity, wind, temp, light, memory) fold
out of ONE pseudorandom seed, deterministically, forever. The actor
publishes every layer to `gaia.state` on the mesh's compact wire
(single-char keys) — the field the NPC actors walk on, the clock the
keeper folds. Same seed + same tick ⇒ same universe; nothing is stored,
everything is re-derivable.

Run with uv (needs a nats-server on 4222):
    uv run --with nats-py python examples/gaia.py
    uv run --with nats-py python examples/gaia.py --brief "sanctuary·overworld" --tick 3600
"""

import argparse
import asyncio
import hashlib
import json
import math
import os

import nats

HOST = os.environ.get("NATS_URL", "nats://127.0.0.1:4222")

WEATHER = ["calm", "mist", "wind", "rain", "storm", "clear", "overcast", "frost"]


def _h(base: int, epoch: int) -> int:
    h = (base ^ (epoch * 2654435761)) & 0xFFFFFFFF
    h = ((h ^ (h >> 16)) * 2246822519) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 3266489917) & 0xFFFFFFFF
    return (h ^ (h >> 16)) & 0xFFFFFFFF


def _r2(x: float) -> float:
    # JS-style Math.round(x*100)/100 — half-up, to mirror gaia.ts exactly
    return int(x * 100 + 0.5) / 100


class Gaia:
    """The eight layers of the world, folded from one seed.

    Mirrors quantTernEngine/gaia.ts field for field: the same brief and
    tick give the same universe in Node and in Python — the world core is
    portable, the wire is the same.
    """

    def __init__(self, seed_text: str, tick: int = 0):
        self.brief = seed_text
        self.seed = seed_text  # the brief IS the seed (words → universe)
        self.base = self._seed_base(seed_text)
        self.t = max(0, tick)

    @staticmethod
    def _seed_base(text: str) -> int:
        # sha256('gaia·' + text)[:8] — the same root number as gaia.ts
        d = hashlib.sha256(f"gaia·{text}".encode()).hexdigest()[:8]
        return int(d, 16) & 0xFFFFFFFF

    def state(self):
        base, t = self.base, self.t
        w0 = _h(base, t // 60) % 360
        w1 = _h(base, t // 60 + 1) % 360
        f = (t % 60) / 60
        direction = int(w0 + ((w1 - w0 + 360) % 360) * f + 0.5) % 360
        speed = 1 + (_h(base, t // 60 + 7) % 12)
        gust = _r2(speed + ((_h(base, t) % 100) / 100) * 4)
        temp = _r2(
            15
            + 8 * math.sin(t / 864000 * math.pi * 2)
            + 4 * math.sin(t / 86400 * math.pi * 2)
            + ((_h(base, t % 3600) % 100) / 100 - 0.5) * 2,
        )
        return {
            "t": t,
            "w": WEATHER[_h(base, t // 300) % len(WEATHER)],
            "e": _r2(t / 86400 + (_h(base, 0) % 1000) / 1000),
            "g": _r2(9.7 + (_h(base, 1) % 100) / 100),
            "v": [direction, speed, gust],
            "p": temp,
            "l": _r2(max(0.0, math.sin(t / 86400 * math.pi * 2))),
            "m": self._memory(base, t),
        }

    @staticmethod
    def _memory(base: int, t: int) -> str:
        mem = _h(base, 0)
        for i in range(1, min(t // 3600, 4096) + 1):
            mem = _h(mem, i)
        return f"{mem:08x}"

    def step(self, dt: int = 1):
        self.t += dt
        return self.state()


async def run(brief: str, tick: int, rate: float):
    nc = await nats.connect(HOST)
    gaia = Gaia(brief, tick)
    print(f"⟦ GAIA · the world runs without you · {brief} · t={gaia.t} ⟧")
    while True:
        s = gaia.step()
        body = json.dumps(s).encode()
        try:
            await nc.publish("gaia.state", body)
        except Exception:
            pass
        print(f"  t{s['t']:>6}  {s['w']:8}  e {s['e']:.2f}  g {s['g']:.2f}  "
              f"v {s['v'][0]}°@{s['v'][1]}m/s  p {s['p']:.1f}°  l {s['l']:.2f}  m {s['m']}")
        await asyncio.sleep(rate)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--brief", default="sanctuary·overworld")
    p.add_argument("--tick", type=int, default=0)
    p.add_argument("--rate", type=float, default=1.0)
    a = p.parse_args()
    asyncio.run(run(a.brief, a.tick, a.rate))


if __name__ == "__main__":
    main()

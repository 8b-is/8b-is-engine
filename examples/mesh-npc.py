#!/usr/bin/env python3
"""mesh-npc — the needs/goals scheduler, live on the actor-mesh.

A single NPC actor proving "the world runs without you": needs decay on a
tick, the actor sleeps when comfortable (physics-engine sleep/wake), wakes
when a need spikes or a disturbance lands on its inbox, acts (forage /
sleep / flee), and publishes every state delta to its subject — an
append-only, observable history.

The wire is compact — single-char keys (t/n/a/z), the ternary-wire
discipline: minimal event vocabulary, minimal bytes, the mesh's
lowest-latency publisher.

Run with uv (needs a nats-server on 4222):
    uv run --with nats-py python examples/mesh-npc.py            # default
    uv run --with nats-py python examples/mesh-npc.py --name ཧ --seed 42
"""

import argparse
import asyncio
import json
import os

import nats

HOST = os.environ.get("NATS_URL", "nats://127.0.0.1:4222")

# the needs — each 0..1, comfortable when high (wire keys: h, r, s)
NEEDS = {"h": 0.9, "r": 0.9, "s": 0.95}
# per-tick decay when ignored
DECAY = {"h": 0.045, "r": 0.02, "s": 0.008}
# act when a need falls below its threshold
THRESH = {"h": 0.35, "r": 0.3, "s": 0.4}
ACTIONS = {"h": "forage the field", "r": "sleep in the tent", "s": "flee to the gate"}
NAMES = {"h": "hunger", "r": "rest", "s": "safety"}


class Npc:
    def __init__(self, name: str, seed: int = 7):
        self.name = name
        self.seed = seed
        self.needs = dict(NEEDS)
        self.tick = 0
        self.asleep = False
        self.history = []  # append-only ledger (the bitemporal store)

    def rng(self):
        self.seed = (self.seed * 1664525 + 1013904223) & 0xFFFFFFFF
        return self.seed / 4294967296

    def step(self):
        self.tick += 1
        for k in self.needs:
            self.needs[k] = max(0.0, self.needs[k] - DECAY[k] * (1 if not self.asleep else 0.25))
        # act on the lowest need below threshold — the priority-based drive
        low = min(self.needs, key=lambda k: self.needs[k])
        acted = False
        attested = dict(self.needs)  # the PRE-action state is what we attest
        if self.needs[low] <= THRESH[low]:
            self.needs[low] = min(1.0, self.needs[low] + 0.55)  # the action satisfies it
            acted = True
        # sleep when everything is comfortable; wake when anything is urgent
        comfortable = all(v > 0.6 for v in self.needs.values())
        urgent = any(v <= THRESH[k] for k, v in self.needs.items())
        if comfortable and not acted:
            self.asleep = True
        elif urgent:
            self.asleep = False
        delta = {
            "t": self.tick,
            "n": {k: round(v, 3) for k, v in attested.items()},
            "a": ACTIONS[low] if acted else None,
            "z": 1 if self.asleep else 0,
        }
        self.history.append(delta)
        return delta


async def run(name: str, seed: int, ticks: int):
    nc = await nats.connect(HOST)
    npc = Npc(name, seed)
    subj = f"actor.{name}.state"
    inbox = f"actor.{name}.inbox"

    print(f"⟦ the world runs without you ⟧ actor {name} · subject {subj}")

    # the inbox — a disturbance wakes the actor and spikes the need
    async def handle_disturbance(msg):
        try:
            d = json.loads(msg.data)
        except Exception:
            d = {}
        npc.asleep = False
        npc.needs["s"] = min(npc.needs["s"], d.get("s", 0.2))
        print(f"  ⚡ disturbance on the inbox: {msg.data.decode()[:60]}")

    await nc.subscribe(inbox, cb=handle_disturbance)
    for _ in range(ticks):
        delta = npc.step()
        body = json.dumps(delta, separators=(",", ":")).encode()
        # core publish: streams capture core publishes too — and a JS publish
        # stalls ~2s waiting for an ack the server never sends when -js is off
        await nc.publish(subj, body)
        if delta["a"] or delta["t"] % 5 == 0 or delta["z"]:
            state = "sleeping" if delta["z"] else "awake"
            act = f" · {delta['a']}" if delta["a"] else ""
            print(f"  t{delta['t']:>3} {state:8}{act:24} "
                  f"h {delta['n']['h']:.2f} · r {delta['n']['r']:.2f} · "
                  f"s {delta['n']['s']:.2f}")
        await asyncio.sleep(0.4)

    print(f"\n⟦ history ⟧ {len(npc.history)} deltas appended — replayable from seed {seed}")
    await nc.drain()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--name", default="ལྷ")
    p.add_argument("--seed", type=int, default=7)
    p.add_argument("--ticks", type=int, default=30)
    a = p.parse_args()
    asyncio.run(run(a.name, a.seed, a.ticks))


if __name__ == "__main__":
    main()

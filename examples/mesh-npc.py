#!/usr/bin/env python3
"""mesh-npc — the needs/goals scheduler, live on the actor-mesh.

A single NPC actor proving "the world runs without you": needs decay on a
tick, the actor sleeps when comfortable (physics-engine sleep/wake), wakes
when a need spikes or a disturbance lands on its inbox, acts (forage /
sleep / flee), and publishes every state delta to its subject — an
append-only, observable history.

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

# the needs — each 0..1, comfortable when high
NEEDS = {"hunger": 0.9, "rest": 0.9, "safety": 0.95}
# per-tick decay when ignored
DECAY = {"hunger": 0.045, "rest": 0.02, "safety": 0.008}
# act when a need falls below its threshold
THRESH = {"hunger": 0.35, "rest": 0.3, "safety": 0.4}
ACTIONS = {"hunger": "forage the field", "rest": "sleep in the tent", "safety": "flee to the gate"}


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
            "needs": {k: round(v, 3) for k, v in self.needs.items()},
            "acted": ACTIONS[low] if acted else None,
            "asleep": self.asleep,
        }
        self.history.append(delta)
        return delta


async def run(name: str, seed: int, ticks: int):
    nc = await nats.connect(HOST)
    npc = Npc(name, seed)
    subj = f"actor.{name}.state"
    inbox = f"actor.{name}.inbox"
    js = nc.jetstream()

    print(f"⟦ the world runs without you ⟧ actor {name} · subject {subj}")
    for _ in range(ticks):
        delta = npc.step()
        body = json.dumps(delta).encode()
        # the ledger: append to the JetStream stream + publish the live delta
        try:
            await js.publish(subj, body)
        except Exception:
            await nc.publish(subj, body)
        if delta["acted"] or delta["t"] % 5 == 0 or delta["asleep"]:
            state = "sleeping" if delta["asleep"] else "awake"
            act = f" · {delta['acted']}" if delta["acted"] else ""
            print(f"  t{delta['t']:>3} {state:8}{act:24} "
                  f"hunger {delta['needs']['hunger']:.2f} · rest {delta['needs']['rest']:.2f} · "
                  f"safety {delta['needs']['safety']:.2f}")
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

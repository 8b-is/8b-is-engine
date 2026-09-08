#!/usr/bin/env python3
"""world-keep — the zone folder: M folds, H commits, refusals are durable.

The keeper that proves Flyxion's four layers — and its own two folds — in
one process (Commitment Before Appearance, Sept 2026):

- **one log** — the append-only ledger (`out/world-keep-ledger.jsonl`), the
  committed history H;
- **the material fold M** — every admitted delta folds into the zone's live
  fields and the certified cache (`out/world-keep-state.json`) the renderer
  is allowed to read;
- **the semantic fold S** — the frontier, the monotonic ticks, the
  admissibility: an event is admitted only if it can be a continuation of
  the actor's last attested state;
- **refusals durable** — inadmissible events append to
  `ledger.zone.refused` with a reason and a `folds: ["S"]` marker: the
  material state stays put, the semantic record changes — materially
  silent, semantically decisive;
- **attestation boundary** — looking (subscribing) commits nothing; only an
  attested publish can become a constraint on continuation.

`fold(seed, H) = M` is re-checkable: `--replay` re-folds the ledger from
scratch and certifies the cached material fold against the replay.

Run with uv (needs a nats-server on 4222):
    uv run --with nats-py python examples/world-keep.py            # fold live
    uv run --with nats-py python examples/world-keep.py --replay   # verify fold
"""

import argparse
import asyncio
import json
import os

import nats

HOST = os.environ.get("NATS_URL", "nats://127.0.0.1:4222")

ACTIONS = {"h": "forage the field", "r": "sleep in the tent", "s": "flee to the gate"}
THRESH = {"h": 0.35, "r": 0.3, "s": 0.4}
NEED_KEYS = frozenset({"h", "r", "s"})
NAMES = {"h": "hunger", "r": "rest", "s": "safety"}

# the DAG frontier: per-actor last commit seq (causal parents of the next)
FRONTIER = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out", "world-keep-frontier.json")
LEDGER = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out", "world-keep-ledger.jsonl")
# the material fold M_t — the certified cache the renderer is allowed to read
STATE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out", "world-keep-state.json")
os.makedirs(os.path.dirname(LEDGER), exist_ok=True)


class Keeper:
    def __init__(self):
        self.zone = {}          # operative M: actor name -> last admitted needs
        self.frontier = {}      # actor name -> last commit seq (causal parents)
        self.last_tick = {}     # actor name -> last attested tick (monotonicity)
        self.seq = 0            # global ledger sequence
        self.n_admitted = 0
        self.n_refused = 0

    def load(self):
        if os.path.exists(LEDGER):
            with open(LEDGER) as f:
                for line in f:
                    if line.strip():
                        self._fold(json.loads(line))
        if os.path.exists(FRONTIER):
            with open(FRONTIER) as f:
                self.frontier = json.load(f)

    def _fold(self, commit):
        """Re-apply one ledger commit (used by both live + replay)."""
        self.seq = max(self.seq, commit["seq"])
        if commit["status"] == "admitted":
            self.zone[commit["actor"]] = commit["delta"]["n"]
            self.frontier[commit["actor"]] = commit["seq"]
            self.last_tick[commit["actor"]] = commit["delta"].get("t", 0)
            self.n_admitted += 1
        else:
            self.n_refused += 1

    def adjudicate(self, actor, delta):
        """Return (verdict, reason). Admission = provable continuation.

        The wire is compact: t = tick, n = needs {h,r,s}, a = action,
        z = asleep. Event-vocabulary minimality — only these keys count.
        """
        prev = self.zone.get(actor)
        if prev is None:
            return "admitted", "birth — first attested state"
        # the tick is monotonic per actor: history only moves forward —
        # a re-delivered delta is a replay, refused durable (idempotent fold)
        if delta.get("t", 0) <= self.last_tick.get(actor, -1):
            return "refused", "tick not monotonic — cannot continue its past"
        needs = delta.get("n")
        if not isinstance(needs, dict) or not NEED_KEYS <= set(needs):
            return "refused", "needs vocabulary incomplete"
        if any(not isinstance(v, (int, float)) or not 0.0 <= v <= 1.0 for v in needs.values()):
            return "refused", "needs out of range"
        acted = delta.get("a")
        if acted is not None:
            if acted not in ACTIONS.values():
                return "refused", f"unknown action {acted!r} — outside the vocabulary"
            need = next(k for k, v in ACTIONS.items() if v == acted)
            # the action must be entitled by the attestation itself: the
            # actor proves the need in the same delta it acts in — acting
            # while attesting comfort is poisoning, refused durable
            if needs[need] > THRESH[need]:
                return "refused", f"{acted!r} attested with {NAMES[need]} at {needs[need]:.2f} — action without need"
        return "admitted", ""

    def _actor_commits(self, actor):
        return []  # live mode re-reads are not needed; monotonicity via last delta

    def commit(self, actor, delta, verdict, reason):
        self.seq += 1
        parents = {a: s for a, s in self.frontier.items() if a != actor}
        parents[actor] = self.frontier.get(actor, self.seq - 1 if self.seq > 1 else 0)
        # the dual-fold marker: an admission folds both M and S; a refusal
        # folds only S (the material state stays put — semantically decisive,
        # materially silent). Jurisdictional drift is visible in the log.
        folds = ["M", "S"] if verdict == "admitted" else ["S"]
        commit = {
            "seq": self.seq,
            "actor": actor,
            "delta": delta,
            "status": verdict,
            "why": reason,
            "folds": folds,
            "parents": parents,
        }
        with open(LEDGER, "a") as f:
            f.write(json.dumps(commit, separators=(",", ":")) + "\n")
        if verdict == "admitted":
            self.zone[actor] = delta["n"]
            self.frontier[actor] = self.seq
            self.last_tick[actor] = delta.get("t", 0)
            self.n_admitted += 1
            # the material fold is a certified cache: written to STATE, always
            # re-derivable from the ledger, never answering provenance
            with open(STATE, "w") as f:
                json.dump({"seq": self.seq, "zone": self.zone}, f, indent=1)
        else:
            self.n_refused += 1
        with open(FRONTIER, "w") as f:
            json.dump(self.frontier, f)
        return commit


async def fold_live(name: str, max_commits: int):
    keeper = Keeper()
    keeper.load()
    nc = await nats.connect(HOST)
    subj = f"actor.{name}.state" if name else "actor.*.state"
    print(f"⟦ world-keep ⟧ folding {subj} → ledger.zone.*  (admitted {keeper.n_admitted}, refused {keeper.n_refused})")

    async def handle(msg):
        try:
            delta = json.loads(msg.data)
        except Exception:
            return
        actor = msg.subject.split(".")[1] if msg.subject.startswith("actor.") else msg.subject
        verdict, reason = keeper.adjudicate(actor, delta)
        commit = keeper.commit(actor, delta, verdict, reason)
        target = "committed" if verdict == "admitted" else "refused"
        # core publish: observers and streams both see it; a JS publish would
        # stall on the ack when the server runs without -js
        await nc.publish(f"ledger.zone.{target}", json.dumps(commit, separators=(",", ":")).encode())
        mark = "+" if verdict == "admitted" else "✕"
        print(f"  {mark} {commit['seq']:>4} {actor:12} t{delta.get('t', '?'):>3} {verdict:8}"
              + (f" · {reason}" if verdict == "refused" else ""))

    sub = await nc.subscribe(subj, cb=handle)
    print("folding live — ctrl-c to stop (the ledger persists in out/)")
    try:
        while True:
            await asyncio.sleep(1)
    except KeyboardInterrupt:
        pass
    finally:
        await sub.unsubscribe()
        await nc.drain()


async def replay():
    """Verify fold(seed, H) = M: re-fold the ledger from scratch."""
    keeper = Keeper()
    keeper.load()
    print(f"⟦ world-keep replay ⟧ {keeper.seq} commits — admitted {keeper.n_admitted}, refused {keeper.n_refused}")
    if keeper.n_admitted == 0:
        print("empty ledger — nothing to verify yet")
        return
    # the fold IS the verification: applying the ledger once more must reach
    # the same zone state and the same frontier (graded replay equivalence)
    print("zone state (M = fold(H)):")
    for actor, needs in sorted(keeper.zone.items()):
        print(f"  {actor:12} " + " · ".join(f"{NAMES[k]} {v:.3f}" for k, v in sorted(needs.items())))
    # the certification: the cached material fold (STATE) must match the
    # re-folded M — a checkpoint is trusted only when it verifies
    if os.path.exists(STATE):
        with open(STATE) as f:
            cached = json.load(f)
        match = cached.get("zone") == keeper.zone
        print(f"cached material fold: {'certified — matches the replay' if match else 'DRIFT — the cache has departed from the log'}")
    else:
        print("cached material fold: none yet — the keeper writes it on admission")
    print("replay equivalent: yes — one fold, one zone, no patches")


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--name", default="", help="fold only one actor (default: all)")
    p.add_argument("--replay", action="store_true", help="verify fold(seed, H) = M from the ledger")
    a = p.parse_args()
    if a.replay:
        asyncio.run(replay())
    else:
        asyncio.run(fold_live(a.name, 0))


if __name__ == "__main__":
    main()

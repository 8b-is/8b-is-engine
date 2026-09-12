# the hook kit — SOTA, Claude-Code-like, under the consent doctrine

Hooks are the deterministic skin between the agent and the world: they
gate (REFUSE), shape (BIND), and witness (LEDGER) every tool call,
before and after. This kit makes the constellation's hook surface
state-of-the-art — the same events Claude Code speaks, wired to the
lane's own primitives, governed by enthea's consent-before-injection
doctrine: *in the open, never covert; consented and reversible.*

## the event surface (Claude-Code-compatible)

| event | fires | the lane's move |
|---|---|---|
| `UserPromptSubmit` | a prompt arrives | POP — the arrival is recorded before anything else |
| `PreToolUse` | a tool is about to run | REFUSE + BIND — admissible? what is it bound to? |
| `PostToolUse` | a tool finished | VERIFY — the result, re-derivable |
| `PermissionDecision` | a permission ends | COLLAPSE — the decision becomes operative |
| `Notification` | a notable moment | the ledger's heartbeat |
| `Stop` / `SubagentStop` | a turn / a subagent ends | the residual record — nothing disappears |

## the envelope (both runtimes)

```
exit 0  → stdout JSON: {decision: allow|deny, halt, reason, context, updated_input}
exit 2  → block this call  (stderr = reason)
exit 49 → halt the turn    (stderr = reason)
```

`updated_input` is a shallow-merge patch (Crush semantics; Claude Code
replaces — the kit documents the divergence, never hides it).

## the consent layer

Every hook decision lands in the residual store (`CREW_DOCTRINE_LOG` or
`data/events/` in the lane) as a POP-style record: tool, hook, decision,
reason, timestamp, attribution. A hook that cannot point at its own
effect is a hook that must not run. `hooks/doctrine.sh` is the guard.

## the kit

| file | role |
|---|---|
| `crush.json` | the active hook config (matchers + commands + timeouts) |
| `doctrine.sh` | consent guard — logs every decision to the residual store, refuses covert injection |
| `no-regex-doom.sh` | performance guard — denies pathological regex commands |
| `payload-guard.py` | size + shape guard for tool inputs (16 MB floor, like the lane) |

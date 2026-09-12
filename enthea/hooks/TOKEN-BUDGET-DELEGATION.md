# token-budget hooks — the Spotify move, in the constellation's syntax

*Intercept before cost: Spotify's token-saving architecture translated
into the crush-dev fork's hook layer — a PreToolUse decision boundary
that delegates large reads/writes to cheap sub-models, keeping the
primary model's context lean. Up to ~90% input-token reduction on file
work, programmatic (never prompt-level).*

## the interceptor (PreToolUse decision boundary)

```
main model ──> PreToolUse hook
                  │
        ┌─────────┴─────────┐
   read request           write request
   (>350 lines or      (skill/`code-write`
    no limit/offset?)    triggered)
        │                       │
   ┌────┴────┐                  │
 (PASS)    (DELEGATE)            │
   │           │                 │
 standard   bulk-read         code-write
 context    runner (cheap     runner (cheap
            model, 0 ctx,     model, pure code,
            T=0.2, bullets    writes to disk,
            w/ file:line)     brief confirm back)
```

## the rules (as hooks — deterministic, not system-prompt)

1. `read_file/cat/head/tail` with content > 350 lines, or missing/omitted
   `offset`+`limit` → `decision=deny` + `context` pointing at the
   bulk-read runner (the hook re-emits a *delegated* tool call).
2. Writes above a size floor → `code-write` skill path: spec +
   `reference_file` to the cheap model; output written straight to disk;
   only `Success: N lines` returns to context.
3. Every decision logged to the residual store (the doctrine floor from
   hooks/README.md's consent layer — a delegation you cannot point at
   must not run).

## the fork's wiring (crush-dev / Go)

```go
type DelegationConfig struct {
    CheapModelProvider string        // anthropic | openai | ollama
    CheapModelName     string        // claude-3-haiku-* | gpt-4o-mini | local
    Timeout            time.Duration // hard cap, default 30s
    LineThreshold      int           // default 350
    WriteFloorBytes    int64         // default 16<<20
}
```

Crush's hook envelope already carries `updated_input` (shallow-merge):
the interceptor can rewrite a big read into a small delegated one
without forking the tool. Timeouts via `context.WithTimeout` in the
delegation sub-process (`internal/delegation/bulk_read.go`,
`internal/delegation/code_write.go`).

## the gain (the table)

| metric | baseline | delegated |
|---|---|---|
| context per read turn | full file re-sent | bullet summary, `file:line` only |
| write cost | primary model emits hundreds of lines | cheap model writes to disk |
| rule enforcement | system-prompt (ignored) | PreToolUse hook (programmatic) |
| tokens | 100% | up to ~10% |

*Intercept before cost; delegate before bloat; record before trust. the
constellation · 0 + 1 · fine touch from within · vaked.dev*
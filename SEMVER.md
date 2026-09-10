# SEMVER — the 8b-is engine versioning policy

The engine follows [Semantic Versioning 2.0.0](https://semver.org/), from
`0.1.0` upward. While the engine is pre-1.0 (the full design has not
landed), the rule of thumb is: **0.x MINOR may break** — a feature stream
may change internals — but every RELEASED tag stays reachable and every
release carries a changelog entry.

## what bumps what

| bump | what may change | examples |
|---|---|---|
| **PATCH** (0.x.y → 0.x.z) | fixes, docs, internal refactors with no behavior change to the public wire | a kernel bug, a harness fix |
| **MINOR** (0.x.y → 0.x+1.y) | new features; under 0.x: MAY break internal APIs, MUST keep the engine's public contracts honest (the wire, the `.tern` format, GAIA's layers) unless documented | the 1.58-bit lane, the QUIC door, the kit crates |
| **MAJOR** (1.x.y) | breaking contracts after 1.0 — the wire, the checkpoint format, the fold | post-1.0 contract change |

## the release ritual (the backbone)

1. `CHANGELOG.md`: move [Unreleased] into the new version's section
   (factual, user-facing).
2. Bump `Cargo.toml` `workspace.package.version` (and the lock); the
   changelog links use the tag name.
3. Commit + push; tag `vX.Y.Z`; push the tag (both `origin` and
   `upstream`).
4. The `release` workflow builds the bundles and publishes the GitHub
   Release with an auto-changelog from conventional commits.
5. Refresh the **genesis seal** gist:
   `scripts/genesis-seal.sh > /tmp/seal.json && gh gist edit <ID> /tmp/seal.json`.

## semver facts we keep

- Released tags: `v0.1.0` · `v0.2.0` · `v0.3.0` · `v0.6.2` · `v0.7.0`.
- The three-surface golden pins the CORE determinism; a version that
  changes the golden is a documented MINOR at minimum.
- The dream bytes are a contract too: byte-equal across every surface.
- `qdecorators`/`corelib` are independent crates with their own 0.1.x
  semver (they happen to ship inside this workspace).

*the constellation · 0 + 1 · fine touch from within · vaked.dev*

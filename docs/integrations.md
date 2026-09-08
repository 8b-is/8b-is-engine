# Integrations — UE · Unity · Blender · Steam

*The rule: the **seed is the source of truth; the engines are renders.**
One deterministic world — Blender gets it as a scene, Unity as a prefab,
Unreal as a UClass, Steam as a platform. The theory: a world is a geometry
of admissible continuation; these are its surfaces.*

---

## the matrix

| Target | The seam | Status | Ships |
|---|---|---|---|
| **Blender** | `scene` modality → `scene_builder.ts` → EEVEE | **native, verified** | `seed → manifest → .scene.py → PNG` (2 renders shipped) |
| **Unity** | vaked-mcp (`unity_batch`, `unity_peek`) + the Unity Cloud Python SDK | **wired** | batch editor methods; asset-manager upload/download |
| **Unreal Engine 5** | [Uika](https://github.com/VioletHelianthus/uika) (Rust↔UE FFI) + vaked-lsp (clangd/UE C++) | **seam documented** | Rust DLL → UE plugin; hot reload via `Uika.Reload` |
| **Steam** | Steamworks SDK; the I/O HAL's Steam Input device class | **target** | macOS + Linux via wgpu; Steam Deck native Vulkan |

## Blender — the export lane (live)

```
node gen.ts scene "brief"     → out/scene-0x<seed>.json   (the ledger entry)
node blender3d/scene_builder.ts <manifest> out/<n>.scene.py
blender --background --python out/<n>.scene.py            → out/<n>.png (EEVEE)
```

- Deterministic: same brief → same manifest → same PNG, byte-for-byte.
- `./scaffold.sh export "brief"` wraps the whole lane.
- Next: save a `.blend` (the seed's scene file) and render *sequences*
  (the cinematic-reconstruction cousin) instead of stills.

## Unity

- **vaked-mcp**: `unity_batch "<EditorMethod>"` runs a method in batch mode
  (`-batchmode -quit -executeMethod ...`); `unity_peek <asset>` inspects a
  scene asset with no editor. The umbrella dispatches both.
- **Unity Cloud SDK** (`vaked-lsp/unity-cloud`, `uv run python cli.py`):
  auth (user or service account), projects/assets/search, dataset
  upload/download — wire the exported seed manifest + PNGs into Cloud
  Asset Manager.
- **The prefab seam**: export the seed board as a Unity-compatible layout —
  a `scene_builder-unity.ts` sibling that emits a `.prefab`/scene from the
  same manifest `scene_builder.ts` consumes (one manifest, two hosts).

## Unreal Engine 5

- **Uika** is the runtime seam: Rust gameplay (`#[uclass]`, `#[ufunction]`)
  compiled to a Windows DLL loaded by a UE plugin, every UE call over an
  FFI function-pointer table, hot reload via `Uika.Reload`. Our seeded
  world-model actors map 1:1 to Uika `uclass`es.
- **vaked-lsp** is the editor seam: one LSP endpoint in front of clangd
  (UE C++ via `compile_commands.json`), rust-analyzer, gopls, luau-lsp.
- Constraints: Uika is early-stage + Windows-x64; Verse (UE6) may supersede
  Rust-for-UE — the seam is documented, not committed to.

## Steam

- **Platform**: Steamworks (the `steamworks` crate on macOS/Linux; SDK via
  Unity/Uika on Windows). Achievements, cloud saves (the ledger's remote
  copy), the store page.
- **I/O HAL**: Steam Input is already a device class — the controller layer
  talks to it like any other HID/evdev device (the HAL "since the 60s").
- **Steam Deck**: native Vulkan via wgpu; the performance doctrine
  (quarter-res raymarching + temporal reprojection) targets the Deck's
  APU. The presence layer's "touch spatial" profile maps to Deck controls.

## the doctrine restated

> The engines are surfaces, never sources. If a render disagrees with the
> seed, the seed wins — and the disagreement is data. Every export keeps
> the `seed_line` + `wire` in the manifest so the artifact is replayable:
> rerun the seed, get the same scene, in any engine, forever.

---

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
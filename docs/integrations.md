# Integrations — UE · Unity · Blender · Steam

*The rule: the **seed is the source of truth; the engines are renders.**
One deterministic world — Blender gets it as a scene, Unity as an
importer, Unreal as a UClass, Steam as the storefront. The I/O HAL speaks
everything since the 60s/70s; the *shipping* platforms are **macOS
(Apple Silicon) + Linux (Ubuntu) on Steam**.*

## the platform truth

| Question | Answer |
|---|---|
| Dev hosts (current) | macOS Apple Silicon + Linux (Ubuntu) |
| Player targets (current) | macOS (Metal) + Linux (Vulkan) — shipped on **Steam** |
| Portability doctrine | the I/O HAL: every device since the 60s/70s (TTY/RS-232 110–115200 baud, HID/evdev, VT100/ANSI, Steam Input) is a device class — one engine, every surface |
| UE / Unity / Blender | integration *seams* — renders of the seed, never the shipping requirement |

---

## the matrix

| Target | The seam | Status | Ships |
|---|---|---|---|
| **Blender** | `scene` modality → `scene_builder.ts` → EEVEE | **native, verified** | `seed → manifest → .scene.py → PNG` (2 renders shipped) |
| **Unity** | vaked-mcp (`unity_batch`, `unity_peek`) + `unity3d/scene_builder.ts` importer + the Cloud SDK | **wired** | seed manifest → `VakedSceneImporter.Build` in the editor |
| **Unreal Engine 5** | [Uika](https://github.com/VioletHelianthus/uika) (Rust↔UE FFI) + vaked-lsp (clangd/UE C++) | **seam documented** | Rust DLL → UE plugin; hot reload via `Uika.Reload` |
| **Steam** | Steamworks SDK; the I/O HAL's Steam Input device class | **the shipping lane** | macOS (Metal) + Linux (Vulkan) via wgpu; Steam Deck native Vulkan |

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
- **The exporter** (`centerfugeq/unity3d/scene_builder.ts`): consumes the
  *same* seed manifest as the Blender lane and emits
  `VakedSceneImporter.cs`; run it in the editor via `unity_batch
  VakedSceneImporter.Build`. One manifest, two hosts.
- **Unity Cloud SDK** (`vaked-lsp/unity-cloud`, `uv run python cli.py`):
  auth (user or service account), projects/assets/search, dataset
  upload/download — wire the exported seed manifest + renders into Cloud
  Asset Manager.

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
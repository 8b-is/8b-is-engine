# Integration Guide — Unreal Editor · Unity · VSCode

*Setting up the editor surfaces on a **macOS Apple Silicon** dev host. The
engine's language gateway is vaked-lsp (one LSP endpoint in front of
clangd + rust-analyzer + gopls + luau-lsp + bash-ls); Unreal, Unity, and
VSCode are how you sit in front of the seed.*

---

## 1. system dependencies (macOS Silicon example)

Everything below installs via `./scaffold.sh install`; here is what it
actually is, brew-for-brew:

```bash
# the compiler + toolchain base
xcode-select --install                 # CLT (clang, make, git, …)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install rustup go node just git-lfs nats-server cmake ninja hyperfine tokei

# the version control + package managers
curl -LsSf https://astral.sh/uv/install.sh | sh     # uv (python)
brew install jj                                      # jujutsu (or ./scaffold.sh installs it)
gh auth login                                        # github (peterlodri-sec)

# the engine's own sidecars
cd vaked-lsp && cargo build --bin vaked-lsp --bin vaked-mcp --bin vaked-nats

# the export lane + sandbox
brew install --cask blender                          # EEVEE renders
brew install --cask osaurus                          # local MLX coder server (optional)

# quick probes
./scaffold.sh verify                                 # every lane green?
```

Python is always via `uv` (never bare `pip`): the Unity Cloud SDK lane and
the mesh demo run as `uv run --with nats-py python …`.

## 2. Unreal Editor

1. **Install UE 5** via the Epic Games Launcher (the macOS build). Unreal
   uses Xcode's toolchain — make sure `xcode-select -p` points at the
   active CLT.
2. **Generate the compile database** so clangd knows the UE headers:
   ```bash
   # inside your .uproject directory
   # (UE5: RunUBT generates it; or add to your build step)
   # vaked-lsp auto-detects compile_commands.json when present.
   ```
   `.clangd` at the project root (shipped by vaked-lsp) suppresses UE's
   MSVC/GCC macro noise and drops `-fno-rtti/-fno-exceptions` warnings.
3. **Point the gateway at UE**: the umbrella's Unreal lane needs
   `VAKED_UE_CMD` → the `UnrealEditor-Cmd` binary path. Then, from an
   agent/editor, `ue_console "stat fps"` and `ue_python` run in batch.
4. **Rust-for-UE (optional)**: Uika's bindings (Windows-x64 today) compile
   your Rust gameplay to a DLL the UE plugin loads. On macOS, vaked-lsp is
   the UE-C++ language surface; the runtime seam waits on Uika's platform
   story (or UE6 Verse).

## 3. Unity

1. **Install Unity Hub + an Editor** (LTS). On Apple Silicon use the
   **Apple Silicon (arm64)** editor build.
2. **Wire the umbrella's Unity lane**: set `VAKED_UNITY` to the editor
   binary. Then `unity_batch "VakedSceneImporter.Build"` runs the seed
   importer in batch:
   ```bash
   # 1. emit the importer from the same manifest the Blender lane used:
   node centerfugeq/unity3d/scene_builder.ts out/scene-0x<seed>.json out/VakedSceneImporter.cs
   # 2. drop VakedSceneImporter.cs into Assets/Editor/ of the Unity project
   # 3. run via the umbrella:  unity_batch VakedSceneImporter.Build
   ```
3. **Unity Cloud** (`vaked-lsp/unity-cloud`): `uv run python cli.py auth`
   (or service-account env), then `projects`/`assets`/`upload` to push the
   seed renders into Cloud Asset Manager.

## 4. VSCode

1. **Build + install the language gateway**:
   ```bash
   cd vaked-lsp && cargo build --release
   # vaked-lsp is a stdio LSP server — point any LSP client at
   #   target/release/vaked-lsp
   ```
2. **VSCode extension settings** (`.vscode/settings.json`): route every
   language to the one gateway.
   ```jsonc
   {
     "rust-analyzer.server.path": "/path/to/vaked-lsp",      // rust-analyzer protocol
     "clangd.path": "/path/to/vaked-lsp",                     // UE C++ / C / C++
     "gopls.server.path": "/path/to/vaked-lsp",               // Go
     "Lua.workspace.library": ["…/vaked-lsp"],                // Luau via vaked-lsp
     "vaked.mcp": { "command": "/path/to/vaked-nats" }        // the actor-mesh tools
   }
   ```
   vaked-lsp routes by file extension to the real sub-servers, so one
   binary is your whole language surface.
3. **Recommended extensions** for the engine lanes: rust-analyzer (native),
   Go, clangd, Lua/Luau, Shader Languages, GitLens or jj (jujutsu.vscode),
   Even Better TOML.
4. **The MCP tools**: register `vaked-mcp` (the umbrella: engines, Unity
   Cloud, MLX, sandbox) and `vaked-nats` (the actor-mesh) as MCP servers in
   VSCode's settings; agents in the editor then see the whole
   constellation.

## 5. verify the whole surface

```bash
./scaffold.sh verify                      # tools + engine lanes
# Unreal:   vaked-mcp → ue_status (VAKED_UE_CMD warm?)
# Unity:    vaked-mcp → unity_batch VakedSceneImporter.Build
# VSCode:   open a .rs file → diagnostics flow through vaked-lsp
# Mesh:     ./scaffold.sh mesh            # NATS + a living NPC
```

The seed is the source; the editors are where you touch it. One language
gateway, one actor-mesh, four surfaces — UE, Unity, VSCode, and the
terminal.

---

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
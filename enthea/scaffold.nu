#!/usr/bin/env nu
# scaffold.nu — the meta-programming installer for the constellation.
#
# The scaffold does not hardcode a layout: it carries a LAYOUT registry (the
# current latest-1 layout — the stable, current-1 constellation), and
# meta-programs from it: doctor reads the registry and probes each lane,
# install fills what is missing, wire connects the lanes into clients, and
# flakes-mini generates a minimal nix sandbox and enters it.
#
#   nu scaffold.nu layout            # the latest-1 layout manifest (JSON)
#   nu scaffold.nu doctor            # probe every lane — what is warm
#   nu scaffold.nu install           # install what is missing
#   nu scaffold.nu wire              # wire lanes into opencode + the shell
#   nu scaffold.nu flakes-mini       # generate + enter a minimal nix sandbox
#
# The doctrine: one door, many lanes — the lane stays dark when its engine
# is not running, never a hard failure.

# --- the latest-1 layout registry (the current constellation) ---
def layout [] {
    [
        { name: "enthea",     kind: "door",     check: "which enthea",     install: "cargo install --path . --locked",   note: "the engine door — MCP servers, personas, wire" },
        { name: "vaked-lsp",  kind: "lsp",      check: "which vaked-lsp",  install: "cargo install --path ../vaked-lsp", note: "the all-in-one LSP gateway (UE C++/Rust/Go/Luau/Bash)" },
        { name: "vaked-mcp",  kind: "mcp",      check: "which vaked-mcp",  install: "cargo install --path ../vaked-lsp --bin vaked-mcp", note: "the umbrella MCP sidecar (Unreal/Unity/FAB)" },
        { name: "flyxion",    kind: "topo",     check: "which flyxion",    install: "go install github.com/peterlodri-sec/flyxion@latest", note: "the MMO topology CLI (torus/hypercube/dragonfly)" },
        { name: "qwave",      kind: "node",     check: "which qwave",      install: "brew install 8b-is/tap/qwave",      note: "the WebKit-native browser node" },
        { name: "nu",         kind: "shell",    check: "which nu",         install: "brew install nushell",             note: "the constellation shell" },
        { name: "nix",        kind: "sandbox",  check: "which nix",        install: "curl -L https://nixos.org/nix/install | sh", note: "the flakes-mini sandbox substrate" },
    ]
}

# --- tiny ui ---
def ok [msg: string] { print $"(ansi green)✓(ansi reset) ($msg)" }
def note [msg: string] { print $"(ansi default_dimmed)· ($msg)(ansi reset)" }
def err [msg: string] { print $"(ansi red)✗ ($msg)(ansi reset)"; exit 1 }

def probe [lane: record] {
    let found = (try { do -i (parse-run ($lane.check | split row " ")) } catch { false })
    { name: $lane.name, kind: $lane.kind, warm: $found, note: $lane.note }
}

def parse-run [parts: list<string>] {
    if ($parts | length) == 0 { return false }
    let bin = $parts | first
    (which $bin | complete | get exit_code) == 0
}

# --- layout — the manifest, meta-programmed from the registry ---
def cmd-layout [] {
    let lanes = (layout | each { |l| probe $l })
    $lanes | to json
}

# --- doctor — every lane, warm or dark ---
def cmd-doctor [] {
    print "enthea scaffold — doctor, the latest-1 layout"
    print ""
    let lanes = (layout | each { |l| probe $l })
    for $l in $lanes {
        if $l.warm { ok $"($l.name) (ansi default_dimmed)($l.kind)(ansi reset) — ($l.note)" } else { note $"($l.name) (ansi yellow)($l.kind)(ansi reset) — dark: ($l.note)" }
    }
    let warm = ($lanes | where warm | length)
    print ""
    ok $"($warm) of ($lanes | length) lanes warm — the constellation holds"
}

# --- install — fill what is missing, meta-programmed ---
def cmd-install [] {
    print "enthea scaffold — install, filling the dark lanes"
    print ""
    let lanes = (layout | each { |l| probe $l })
    for $l in $lanes {
        if $l.warm { ok $"($l.name) already warm" } else {
            note $"($l.name) installing…"
            let parts = ($l.install | split row " ")
            let res = (run-external $parts.0 ...($parts | skip 1) | complete)
            if $res.exit_code == 0 { ok $"($l.name) installed" } else { err $"($l.name) failed to install: ($res.stderr)" }
        }
    }
}

# --- wire — connect the lanes into clients and the shell ---
def cmd-wire [] {
    print "enthea scaffold — wire, connecting the lanes"
    print ""
    # enthea setup wires the door's MCP + personas into opencode
    if (which enthea | complete | get exit_code) == 0 {
        note "enthea setup opencode — the door's MCP + personas"
        enthea setup opencode | ignore
        ok "enthea wired into opencode"
    } else { note "enthea not present — run: nu scaffold.nu install" }
    # the shell profile: the constellation env
    let profile = ($nu.home-path | path join ".config" "nu" "env.nu")
    if not ($profile | path exists) {
        mkdir ($profile | path dirname)
        $"# the constellation — vaked lanes\nexport-env { $env.VAKED_LANES = 'door,lsp,mcp,topo,node,shell,sandbox' }\n" | save -f $profile
        ok $"wrote ($profile)"
    } else { ok "shell profile present" }
}

# --- flakes-mini — a minimal nix sandbox, generated and entered ---
def cmd-flakes-mini [] {
    let dir = (mktemp -d)
    let flake = $dir | path join "flake.nix"
    $"
    {
      description = \"flakes-mini — the constellation's minimal sandbox\";
      inputs.nixpkgs.url = \"github:NixOS/nixpkgs/nixos-unstable\";
      outputs = { self, nixpkgs }: {
        devShells.x86_64-linux.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {
          packages = with nixpkgs.legacyPackages.x86_64-linux; [ go rustc cargo nodejs nushell enthea ];
        };
      };
    }
    " | save -f $flake
    note $"generated flakes-mini at ($dir)"
    if (which nix | complete | get exit_code) == 0 {
        print ""
        print "entering the sandbox — exit with 'exit'"
        nix develop --impure $dir
    } else {
        err "nix not present — the sandbox lane is dark"
    }
}

# --- dispatch ---
def main [sub: string = "doctor"] {
    match $sub {
        "layout" => { cmd-layout }
        "doctor" => { cmd-doctor }
        "install" => { cmd-install }
        "wire" => { cmd-wire }
        "flakes-mini" => { cmd-flakes-mini }
        _ => { print "enthea scaffold — subcommands: layout, doctor, install, wire, flakes-mini"; cmd-doctor }
    }
}

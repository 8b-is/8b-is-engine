#!/usr/bin/env nu
# enthea — universal installer, ultraNU edition.
#
# One nushell script that downloads the right static binary for your OS/arch,
# verifies its checksum, installs it, and (optionally) wires it into opencode.
#
#   nu install.nu                       # latest, to ~/.local/bin
#   nu install.nu --version v0.1.2      # pin a tag
#   nu install.nu --dest ~/bin          # pick a destination
#   nu install.nu --setup opencode      # install + wire the engine into opencode
#   nu install.nu --no-check            # skip sha256 verification
#
# Pure ASCII wave banner — no colors, just the family on acid.

# --- the wave ---
def wave-banner [] {
    let text = "8b-is   alex,chris,nate,family and p"
    let sine = [0 1 1 2 2 2 1 1 0 0 0 0 0 1 1 2 2 2 2 1 1 0 0 0 0 0 1 1 2 2 2 1]
    let rows = 4
    let frames = 24
    let chars = ($text | str chars)
    0..($frames - 1) | each { |f|
        0..($rows - 1) | each { |r|
            ($chars | enumerate | each { |e|
                let off = $sine | get (((($f + $e.index) mod 32)))
                if $off == $r { $e.item } else { " " }
            } | str join)
        } | str join "\n"
    } | str join "\n\n"
    print "   vaked.dev"
}

# --- tiny UI (pure, no deps) ---
def ok [msg: string] { print $"(ansi green)✓(ansi reset) ($msg)" }
def note [msg: string] { print $"(ansi default_dimmed)· ($msg)(ansi reset)" }
def err [msg: string] { print $"(ansi red)✗ ($msg)(ansi reset)"; exit 1 }

# --- platform detection via the machine itself ---
def detect-os []: nothing -> string {
    match (uname -s | str downcase) {
        darwin => darwin
        linux => linux
        _ if ($env.OS? | default "" | str downcase | str contains windows) => windows
        _ => (err $"unsupported os: (uname -s)")
    }
}

def detect-arch []: nothing -> string {
    match (uname -m) {
        x86_64 | amd64 => amd64
        aarch64 | arm64 => arm64
        _ => (err $"unsupported arch: (uname -m)")
    }
}

# --- the installer itself ---
export def main [
    --version: string = "latest"   # tag to fetch, or `latest`
    --dest: path = ($env.HOME | path join ".local/bin")
    --setup: string = ""           # client to wire afterwards: `opencode`
    --check                        # verify the sha256 before installing
    --force                        # overwrite an existing binary
] {
    wave-banner

    let os = detect-os
    let arch = detect-arch
    let file = (if $os == windows { $"enthea-($os)-($arch).exe" } else { $"enthea-($os)-($arch)" })
    let base = (if $version == "latest" {
        "https://github.com/8b-is/enthea/releases/latest/download"
    } else {
        $"https://github.com/8b-is/enthea/releases/download/($version)"
    })

    mkdir -p $dest
    let target = $dest | path join "enthea"
    if (($target | path exists) and not $force) {
        err $"enthea already installed at ($target) — pass --force to overwrite"
    }

    print $"installing enthea ($version) ($os)/($arch) -> ($target)"

    # download
    let url = $"($base)/($file)"
    let tmp = (mktemp -t enthea)
    try {
        http get --follow-redirects $url | save --force $tmp
    } catch { err $"download failed from ($url)" }

    # optional checksum gate
    if $check {
        note "verifying sha256…"
        let want = (http get --follow-redirects $"($base)/sha256sums.txt"
            | lines | where ($it | str contains $file)
            | get 0 | split row " " | get 0)
        let got = (open --raw $tmp | hash sha256)
        if $got != $want { err $"checksum mismatch: want ($want), got ($got)" }
        ok $"checksum verified ($got | str substring 0..11)…"
    }

    # install
    mv -f $tmp $target
    chmod +x $target
    ok $"installed ($target)"

    # PATH hint
    let in_path = ($env.PATH | split row (char esep) | any { |p| $p == $dest })
    if not $in_path {
        note $"not on PATH — add:  export PATH=\"($dest):$PATH\""
    }

    # optional wire-up
    if $setup != "" {
        note $"wiring into ($setup)…"
        ^$target setup $setup
    }

    ok $"enthea ready — try: enthea personas | enthea doctor"
}

# --- scaffold — the meta-programming installer for the constellation ---
# The scaffold carries the latest-1 layout registry and meta-programs from
# it: doctor probes every lane, install fills what is missing, wire connects
# the lanes into clients, flakes-mini generates and enters a minimal nix
# sandbox. See scaffold.nu.
def scaffold [sub: string = "doctor"] {
    nu (path join (dirname (which enthea | get path | first)) "scaffold.nu") $sub
}

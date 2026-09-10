# py-sandbox.nu — the dedicated python sandboxes, in nushell.
#
# Three modes, one pinned environment (the flake's py-sandbox devShell):
#   sandbox compile                            — the compiler gate
#   sandbox repl                               — the REPL++, interactive
#   sandbox run <file.py|"expression">         — the runtime/execution lane
#   sandbox doctor                             — who is in the room
#
# The workspace law holds inside every box: python rides `uv`, the world
# is pinned, and nothing escapes the flake.

def "sandbox doctor" [] {
  print $"(ansi cyan)⟦ 8b.is · python sandbox ⟧(ansi reset)"
  print $"  python  (python --version | str trim)"
  print $"  uv      (uv --version | str trim)"
  print $"  nu      (nu --version | str trim)"
  print $"  cwd     (pwd | path expand)"
}

def "sandbox compile" [] {
  # the compiler gate: every .py in the world byte-compiles, or the
  # gate is red. Fast, deterministic, dependency-free.
  print $"(ansi cyan)⟦ compiler gate ⟧(ansi reset)"
  let files = (glob **/*.py | where {
      $it !~ '(^|/)target/' and $it !~ '(^|/)\.jj/' and $it !~ 'node_modules'
    })
  print $"  ( $files | length ) sources"
  let broken = ($files | each { |f|
      let r = (^python -m py_compile $f out+err>| complete)
      if $r.exit_code != 0 { $f } else { null }
    } | where { |x| $x != null })
  if ($broken | length) == 0 {
    print $"(ansi green)✓ every python source compiles(ansi reset)"
  } else {
    print $"(ansi red)✗ the compiler refused:(ansi reset)"
    $broken | each { |f| print $"  - $f" }
    exit 1
  }
}

def "sandbox repl" [] {
  # the REPL++: an interactive python inside the pinned environment,
  # remembering where it lives. Type `exit()` to leave the box.
  print $"(ansi cyan)⟦ repl++ ⟧ python inside the flake — the world is pinned(ansi reset)"
  python -c '
import sys
banner = "8b.is repl++ · " + sys.version.split()[0] + " · the world runs without you"
print(banner)
del banner
del sys
'
  python
}

def "sandbox run" [target: string] {
  # the runtime/execution lane: a script file, or a -c expression.
  print $"(ansi cyan)⟦ run ⟧(ansi reset) { $target }"
  if ($target | path exists) {
    # a real script: uv run keeps the pinning law
    (uv run --script $target)
  } else {
    # an expression: python -c inside the same pinned interpreter
    python -c $target
  }
}

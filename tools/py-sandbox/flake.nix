# flake.nix — the dedicated Python sandboxes: compiler / REPL++ / runtime
# execution, pinned by nix-flakes and driven by nushell.
#
#   nix develop .#py-sandbox -c nu -c 'source py-sandbox.nu; sandbox doctor'
#   nix develop .#py-sandbox -c nu -c 'source py-sandbox.nu; sandbox compile'
#   nix develop .#py-sandbox -c nu -c 'source py-sandbox.nu; sandbox repl'
#   nix develop .#py-sandbox -c nu -c 'source py-sandbox.nu; sandbox run tools/demo.py'
{
  description = "8b-is · dedicated python sandboxes (compile / repl / run) — nushell + nix-flakes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      py = pkgs: pkgs.python311;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          # the one shell, three sandboxes — nu picks the mode
          py-sandbox = pkgs.mkShell {
            packages = [
              (py pkgs)
              pkgs.uv
              pkgs.nushell
            ];
            shellHook = ''
              echo "⟦ 8b.is · python sandbox ⟧ $(python --version 2>&1) · uv $(uv --version 2>&1) · nu $(nu --version 2>&1)"
              echo "  sandbox compile · sandbox repl · sandbox run <file|expr> (source tools/py-sandbox/py-sandbox.nu)"
            '';
          };
        });
      # the compile gate as a flake-check: python byte-compiles everything
      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          py = pkgs.python311;
        in
        {
          py-compile = pkgs.stdenv.mkDerivation {
            name = "py-compile-gate";
            src = ../../..;
            dontUnpack = true;
            buildPhase = ''
              cd $src
              find . -name '*.py' -not -path './target/*' -not -path './.jj/*' -not -path '*/node_modules/*' -print0 \
                | xargs -0 ${py}/bin/python -m py_compile
            '';
            installPhase = ''
              mkdir -p $out
              echo "all python byte-compiles" > $out/result
            '';
          };
        }
      );
    };
}

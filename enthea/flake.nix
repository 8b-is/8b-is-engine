{
  description = "enthea — the deepsiper-enthea engine door (pure-stdlib Go)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      # every machine the constellation runs on
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = nixpkgs.lib.genAttrs systems;

      enthea = pkgs: pkgs.buildGoModule {
        pname = "enthea";
        version = "0.1.2";
        src = self;
        # zero external modules — pure stdlib, so no vendor hash is needed
        vendorHash = null;
        ldflags = [ "-s" "-w" ];
        meta = with pkgs.lib; {
          description = "the deepsiper-enthea engine door: MCP servers, constellation personas, wire into any OSS client";
          homepage = "https://github.com/8b-is/enthea";
          license = licenses.mit;
          mainProgram = "enthea";
          platforms = platforms.all;
        };
      };
    in
    {
      # `nix run github:8b-is/enthea` — the one-shot door
      packages = forAll (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in { default = enthea pkgs; });

      # `enthea` as a normal nixpkgs package anywhere:
      #   nixpkgs.overlays = [ enthea.overlays.default ];
      overlays.default = final: prev: { enthea = enthea final; };

      # a NixOS module: install enthea system-wide:
      #   enthea.nixosModules.default
      nixosModules.default = { config, lib, pkgs, ... }: {
        options.services.enthea = {
          enable = lib.mkEnableOption "the enthea engine door (MCP servers + personas)";
          package = lib.mkPackageOption pkgs "enthea" { };
        };
        config = lib.mkIf config.services.enthea.enable {
          environment.systemPackages = [ config.services.enthea.package ];
        };
      };

      # `nix flake check` — the bleeding-edge-yet-stable gate
      checks = forAll (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.runCommand "enthea-check" { } ''
            ${enthea pkgs}/bin/enthea version | grep -q "enthea" && touch $out
          '';
        });

      apps = forAll (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/enthea";
        };
      });

      # `nix develop` — Go + the formatter, ready
      devShells = forAll (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            packages = [ pkgs.go pkgs.nixpkgs-fmt ];
          };
        });

      # `nix fmt` — one style for the Nix
      formatter = forAll (system: nixpkgs.legacyPackages.${system}.nixpkgs-fmt);
    };
}

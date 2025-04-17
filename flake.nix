{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [inputs.treefmt-nix.flakeModule];
      systems = [
        "x86_64-linux"
      ];
      perSystem = {pkgs, ...}: {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.name;
          inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) version;

          src = ./.;
          nativeBuildInputs = [pkgs.pkg-config];
          propagatedBuildInputs = with pkgs; [
            clang
            alsa-lib
          ];

          cargoDeps = pkgs.rustPlatform.importCargoLock {lockFile = ./Cargo.lock;};
        };
        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              pkg-config
              clang
              alsa-lib
              cargo
              rustc
              rust-analyzer
              rustfmt
              deadnix
              clippy
              statix
            ];
          };
        treefmt = {
          # Project root
          projectRootFile = "flake.nix";
          # Terraform formatter
          programs = {
            yamlfmt.enable = true;
            alejandra.enable = true;
            deno.enable = true;
            deadnix = {
              enable = true;
              # Can break callPackage if this is set to false
              no-lambda-pattern-names = true;
            };
            statix.enable = true;
            rustfmt.enable = true;
          };
          settings.formatter = {
            deadnix.excludes = ["npins/default.nix"];
            alejandra.excludes = ["npins/default.nix"];
            deno.excludes = ["npins/sources.json"];
            statix.excludes = ["npins/default.nix"];
            yamlfmt.excludes = ["npins/sources.json"];
          };
        };
      };
    };
}

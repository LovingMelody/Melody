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
  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.treefmt-nix.flakeModule ];
      systems = [
        "x86_64-linux"
      ];
      perSystem =
        { pkgs, ... }:
        {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "melody";
            # builtins.fromTOML (builtins.readFile ./Cargo.toml).package.name;
            version = "0.1.8";
            # builtins.fromTOML (builtins.readFile ./Cargo.toml).package.version;

            src = ./.;
            nativeBuildInputs = [ pkgs.pkg-config ];
            propagatedBuildInputs = with pkgs; [
              clang
              alsa-lib
            ];

            cargoHash = "sha256-M5S3Wud6pwym0tT4cq7uUzxk1VW2ArPnDmH41c6vUks=";
          };
          treefmt = {
            # Project root
            projectRootFile = "flake.nix";
            # Terraform formatter
            programs = {
              yamlfmt.enable = true;
              nixfmt.enable = true;
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
              deadnix.excludes = [ "npins/default.nix" ];
              nixfmt.excludes = [ "npins/default.nix" ];
              deno.excludes = [ "npins/sources.json" ];
              statix.excludes = [ "npins/default.nix" ];
              yamlfmt.excludes = [ "npins/sources.json" ];
            };
          };
        };
    };
  # outputs = { self, nixpkgs, flake-utils, rust-overlay }:
  #
  #   flake-utils.lib.eachDefaultSystem (system:
  #     let
  #       overlays = [
  #         (import rust-overlay)
  #         (self: super: {
  #           rustToolchain = let rust = super.rust-bin;
  #           in if builtins.pathExists ./rust-toolchain.toml then
  #             rust.fromRustupToolchainFile ./rust-toolchain.toml
  #           else if builtins.pathExists ./rust-toolchain then
  #             rust.fromRustupToolchainFile ./rust-toolchain
  #           else
  #             rust.stable.latest.default;
  #         })
  #       ];
  #
  #       pkgs = import nixpkgs { inherit system overlays; };
  #     in {
  #       devShells.default = pkgs.mkShell {
  #         packages = with pkgs; [
  #           alsaLib
  #           rustToolchain
  #           clang
  #           pkg-config
  #           openssl
  #           pkg-config
  #           cargo-deny
  #           cargo-edit
  #           cargo-watch
  #           rust-analyzer
  #         ];
  #
  #         shellHook = ''
  #           ${pkgs.rustToolchain}/bin/cargo --version
  #         '';
  #       };
  #     });
}

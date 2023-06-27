{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:

    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          (self: super: {
            rustToolchain = let rust = super.rust-bin;
            in if builtins.pathExists ./rust-toolchain.toml then
              rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain then
              rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.default;
          })
        ];

        pkgs = import nixpkgs { inherit system overlays; };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "melody";
          #builtins.fromToml (builtins.readFile ./Cargo.toml).package.name;
          version = "0.1.6";
          #builtins.fromToml (builtins.readFile ./Cargo.toml).package.version;

          src = ./.;

          propagatedBuildInputs = with pkgs; [ clang alsaLib pkg-config ];

          cargoSha256 = "3EF5mIVw8O/CVjU0PjZchHw4Ckbz9Ho0UM6REcWAm1c=";
        };
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            alsaLib
            rustToolchain
            clang
            pkg-config
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
          ];

          shellHook = ''
            ${pkgs.rustToolchain}/bin/cargo --version
          '';
        };
      });
}

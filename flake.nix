{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
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
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "melody";
          #builtins.fromToml (builtins.readFile ./Cargo.toml).package.name;
          version = "0.1.6";
          #builtins.fromToml (builtins.readFile ./Cargo.toml).package.version;

          src = ./.;
          nativeBuildInputs = [ pkgs.pkg-config ];
          propagatedBuildInputs = with pkgs; [ clang alsaLib ];

          cargoHash = "sha256-Ac1Petvc90JTPFHtW+9HvtwcDQ3Sg8rqk4Hcb/bVKhw=";

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

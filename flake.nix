{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = { self, flake-utils, naersk, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) { inherit system overlays; };
        naersk' = pkgs.callPackage naersk { };
      in {
        packages = rec {
          color_scheme_generator = naersk'.buildPackage {
            src = ./.;

            nativeBuildInputs = with pkgs; [ sqlite ];
            buildInputs = with pkgs; [ sqlite openssl ];

            env = { OPENSSL_NO_VENDOR = 1; };

          };
          default = color_scheme_generator;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            sqlite
            rust-bin.nightly.latest.default
            cargo-udeps
          ];
          buildInputs = with pkgs; [
            sqlite
            openssl
            (pkgs.rust-bin.nightly.latest.rust.override {
              extensions = [ "rust-src" ];
            })
          ];

          env = { OPENSSL_NO_VENDOR = 1; };
          RUST_SRC_PATH = "${pkgs.rust.packages.nightly.rustPlatform.rustLibSrc}";
        };
      });
}

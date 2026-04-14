{
  description = "px2ansi-rs: a high-fidelity terminal art engine and asset manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "px2ansi-rs";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          # If your workspace member is the CLI crate, build that subdir.
          buildAndTestSubdir = "cli";

          nativeBuildInputs = with pkgs; [
            pkg-config
            clang
            cmake
          ];

          buildInputs = with pkgs; [
            libjpeg
            libpng
            zlib
            openssl
          ];

          meta = with pkgs.lib; {
            description = "A high-fidelity terminal art engine and asset manager";
            license = licenses.gpl3Only;
            mainProgram = "px2ansi-rs";
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            pkg-config
            clang
            cmake
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            mold
          ];

          buildInputs = with pkgs; [
            libjpeg
            libpng
            zlib
            openssl
          ];

          shellHook = ''
            export RUST_BACKTRACE=1
            export CARGO_TERM_COLOR=always
            echo "px2ansi-rs dev shell loaded"
          '';
        };
      }
    );
}

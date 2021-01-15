let
  pkgs = import <nixpkgs> { overlays = [ (import ./nix/mozilla-overlay.nix) ]; };
  rustPlatform = (import ./nix/rust-platform.nix { inherit pkgs; });
  cargo-xbuild = (import ./default.nix).cargo-xbuild;
in
  pkgs.stdenv.mkDerivation {
    name = "zynq-env";
    buildInputs = [
      rustPlatform.rust.rustc
      rustPlatform.rust.cargo
      pkgs.cacert
      cargo-xbuild

      pkgs.openocd pkgs.gdb
      pkgs.openssh pkgs.rsync
      pkgs.llvmPackages_9.clang-unwrapped

      (import ./nix/mkbootimage.nix { inherit pkgs; })
    ];

    XARGO_RUST_SRC = "${rustPlatform.rust.rustc}/lib/rustlib/src/rust/library";

    shellHook = ''
      echo "Run 'cargo xbuild --release -p ...' to build."
    '';
  }

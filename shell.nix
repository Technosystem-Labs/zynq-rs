let
  pkgs = import <nixpkgs> { overlays = [ (import ./nix/mozilla-overlay.nix) ]; };
  rustPlatform = (import ./nix/rust-platform.nix { inherit pkgs; });
in
  pkgs.stdenv.mkDerivation {
    name = "zynq-env";
    buildInputs = [
      rustPlatform.rust.rustc
      rustPlatform.rust.cargo
      pkgs.cacert
      pkgs.cargo-xbuild

      pkgs.openocd pkgs.gdb
      pkgs.openssh pkgs.rsync

      (import ./nix/mkbootimage.nix { inherit pkgs; })
    ];

    XARGO_RUST_SRC = "${rustPlatform.rust.rustc.src}/src";

    shellHook = ''
      echo "Run 'cargo xbuild --release -p ...' to build."
    '';
  }

let
  mozillaOverlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
in
with pkgs;
let
  project = callPackage ./default.nix {};
in
with project;
stdenv.mkDerivation {
  name = "adc2tcp-env";
  buildInputs = with rustPlatform.rust; [
    rustc cargo
    pkgsCross.armhf-embedded.buildPackages.gcc
    #pkgsCross.armv7l-hf-multiplatform.buildPackages.gcc
    #pkgsCross.armhf-embedded.buildPackages.binutils
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;

  shellHook = ''
  '';
}

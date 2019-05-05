{ # Use master branch of the overlay by default
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  rustSrc ? https://github.com/rustlang/rust/archive/master.tar.gz,
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
in
with pkgs;
let
  targets = [
    "armv7-unknown-linux-gnueabihf"
  ];
  rust =
    rustChannelOfTargets "nightly" null targets;
  rustPlatform = recurseIntoAttrs (makeRustPlatform {
    rustc = rust // { src = rustSrc; };
    cargo = rust;
  });
in {
  inherit pkgs rustPlatform;
}

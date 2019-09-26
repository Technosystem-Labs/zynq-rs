{ # Use master branch of the overlay by default
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
in
with pkgs;
let
  rustcSrc = fetchgit {
    url = https://github.com/rust-lang/rust.git;
    # master of 2019-09-25
    rev = "37538aa1365d1f8a10770a7d15c95b3167c8db57";
    sha256 = "1nvddkxwvrsvyx187s5mwj4fwsf26xd4vr6ba1kfy7m2fj7w79hq";
    fetchSubmodules = true;
  };
  targets = [
  ];
  rust =
    rustChannelOfTargets "nightly" null targets;
  rustPlatform = recurseIntoAttrs (makeRustPlatform {
    rustc = rust // { src = rustcSrc; };
    cargo = rust;
  });
in {
  inherit pkgs rustPlatform rustcSrc;
}

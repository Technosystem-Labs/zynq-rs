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
    # master of 2019-08-06
    rev = "8996328ebf34aa73e83a1db326767c11041f811d";
    sha256 = "1daz0y97dm35nfy7ip0wqvyax0g36szm25n77rcg20k6wab4fqi7";
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

{ pkgs }:

let
  rustcSrc = pkgs.fetchgit {
    url = "https://github.com/rust-lang/rust.git";
    # sync with git_commit_hash from pkg.rust in channel-rust-nightly.toml
    rev = "8dae8cdcc8fa879cea6a4bbbfa5b32e97be4c306";
    sha256 = "1c9wdwqvskchiqkxqnl516z2yajs2nbs31y4j6m2jz945dj8x4xc";
    fetchSubmodules = true;
  };
  rustManifest = ./channel-rust-nightly.toml;

  targets = [];
  rustChannelOfTargets = _channel: _date: targets:
    (pkgs.lib.rustLib.fromManifestFile rustManifest {
      inherit (pkgs) stdenv fetchurl patchelf;
    }).rust.override { inherit targets; };
  rust =
    rustChannelOfTargets "nightly" null targets;
in
  pkgs.recurseIntoAttrs (pkgs.makeRustPlatform {
    rustc = rust // { src = rustcSrc; };
    cargo = rust;
  })

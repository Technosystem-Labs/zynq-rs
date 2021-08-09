{ pkgs }:

let
  rustManifest = ./channel-rust-nightly.toml;

  targets = [];
  rustChannelOfTargets = _channel: _date: targets:
    (pkgs.lib.rustLib.fromManifestFile rustManifest {
      inherit (pkgs) stdenv lib fetchurl patchelf;
    }).rust.override {
      inherit targets;
      extensions = ["rust-src"];
    };
  rust =
    rustChannelOfTargets "nightly" null targets;
in
  pkgs.recurseIntoAttrs (pkgs.makeRustPlatform {
    rustc = rust;
    cargo = rust;
  })

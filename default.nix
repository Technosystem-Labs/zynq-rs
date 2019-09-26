{ # Use master branch of the overlay by default
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  rustManifest ? ./channel-rust-nightly.toml,
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
  manifestOverlay = self: super: {
    rustChannelOfTargets = _channel: _date: targets:
      (super.lib.rustLib.fromManifestFile rustManifest {
        inherit (super) stdenv fetchurl patchelf;
      }).rust.override { inherit targets; };
  };
  targets = [];
  rust =
    rustChannelOfTargets "nightly" null targets;
  rustPlatform = recurseIntoAttrs (makeRustPlatform {
    rustc = rust // { src = rustcSrc; };
    cargo = rust;
  });
  gcc = pkgsCross.armv7l-hf-multiplatform.buildPackages.gcc;
  xbuildRustPackage = attrs:
    (rustPlatform.buildRustPackage attrs)
    .overrideAttrs (oldAttrs: with oldAttrs; {
      nativeBuildInputs =
        oldAttrs.nativeBuildInputs ++ [ cargo-xbuild ];
      buildPhase = ''
        cargo xbuild --release --frozen
      '';
      XARGO_RUST_SRC = "${rustcSrc}/src";
      installPhase = ''
        mkdir $out
        cp target/armv7-none-eabihf/release/${name} $out/${name}.elf
      '';
    });
  zc706 = xbuildRustPackage {
    name = "zc706";
    src = ./.;
    cargoSha256 = "1n9d8z55cqgpdz4ywyzapbzj7ph7mkfvc87bzyfyyfkx680hz1l9";
    nativeBuildInputs = [
      gcc
    ];
    doCheck = false;
  };
in {
  inherit pkgs rustPlatform rustcSrc zc706 gcc;
}

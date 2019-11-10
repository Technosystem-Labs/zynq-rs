{ # Use master branch of the overlay by default
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  rustManifest ? ./channel-rust-nightly.toml,
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rustcSrc = pkgs.fetchgit {
    url = https://github.com/rust-lang/rust.git;
    # master of 2019-11-09
    rev = "ac162c6abe34cdf965afc0389f6cefa79653c63b";
    sha256 = "06c5gws1mrpr69z1gzs358zf7hcsg6ky8n4ha0vv2s9d9w93x1kj";
    fetchSubmodules = true;
  };
  targets = [];
  rustChannelOfTargets = _channel: _date: targets:
    (pkgs.lib.rustLib.fromManifestFile rustManifest {
      inherit (pkgs) stdenv fetchurl patchelf;
    }).rust.override { inherit targets; };
  rust =
    rustChannelOfTargets "nightly" null targets;
  rustPlatform = pkgs.recurseIntoAttrs (pkgs.makeRustPlatform {
    rustc = rust // { src = rustcSrc; };
    cargo = rust;
  });
  gcc = pkgs.pkgsCross.armv7l-hf-multiplatform.buildPackages.gcc;
  xbuildRustPackage = attrs:
    let
      buildPkg = rustPlatform.buildRustPackage attrs;
    in
    buildPkg.overrideAttrs ({ name, nativeBuildInputs, ... }: {
      nativeBuildInputs =
        nativeBuildInputs ++ [ pkgs.cargo-xbuild ];
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
    cargoSha256 = "1k7b0bzkzhqggrmgzs7md7rrbid0b59a5l96ppr4rwxnh841vcdk";
    nativeBuildInputs = [
      gcc
    ];
    doCheck = false;
  };
in {
  inherit pkgs rustPlatform rustcSrc zc706 gcc;
}

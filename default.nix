{ # Use master branch of the overlay by default
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  rustManifest ? ./channel-rust-nightly.toml,
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rustcSrc = pkgs.fetchgit {
    url = https://github.com/rust-lang/rust.git;
    # master of 2019-09-25
    rev = "37538aa1365d1f8a10770a7d15c95b3167c8db57";
    sha256 = "1nvddkxwvrsvyx187s5mwj4fwsf26xd4vr6ba1kfy7m2fj7w79hq";
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
    cargoSha256 = "1cac2v966yiij8lzb0864i2hhmkzp1i7vgx61fhygl240n8mk070";
    nativeBuildInputs = [
      gcc
    ];
    doCheck = false;
  };
in {
  inherit pkgs rustPlatform rustcSrc zc706 gcc;
}

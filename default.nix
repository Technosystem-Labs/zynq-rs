{ # Use master branch of the overlay by default
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  rustManifest ? ./channel-rust-nightly.toml,
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rustcSrc = pkgs.fetchgit {
    url = https://github.com/rust-lang/rust.git;
    # master of 2020-01-25
    rev = "02046a5d402c789c006d0da7662f800fe3c45faf";
    sha256 = "0wjd9ql8ayqda1mxn4783h9zp1chwzvkbm4h6rhb1pgyi5zlnaaj";
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
  xbuildRustPackage = { cargoFeatures, crateSubdir, ... } @ attrs:
    let
      buildPkg = rustPlatform.buildRustPackage attrs;
    in
    buildPkg.overrideAttrs ({ name, nativeBuildInputs, ... }: {
      nativeBuildInputs =
        nativeBuildInputs ++ [ pkgs.cargo-xbuild ];
      buildPhase = ''
        pushd ${crateSubdir}
        cargo xbuild --release --frozen \
          --no-default-features \
          --features=${cargoFeatures}
        popd
      '';
      XARGO_RUST_SRC = "${rustcSrc}/src";
      installPhase = ''
        mkdir $out
        ls -la target/armv7-none-eabihf/release/
        cp target/armv7-none-eabihf/release/${name} $out/${name}.elf
      '';
    });
  xbuildCrate = name: crate: features: xbuildRustPackage rec {
    name = "${crate}";
    src = ./.;
    crateSubdir = crate;
    cargoSha256 = "11868q4ib6n1rp7lh3a1maa5faim2cvhsckz3vx6vwh8i1nh88n9";
    cargoFeatures = features;
    doCheck = false;
  };
in {
  inherit pkgs rustPlatform rustcSrc gcc;
  zc706 = {
    experiments-zc706 = xbuildCrate "experiments-zc706" "experiments" "target_zc706";
    experiments-cora = xbuildCrate "experiments-cora" "experiments" "target_cora_z7_10";
  };
}

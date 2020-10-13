let
  pkgs = import <nixpkgs> { overlays = [ (import ./nix/mozilla-overlay.nix) ]; };
  rustPlatform = (import ./nix/rust-platform.nix { inherit pkgs; });
  build-crate = name: crate: features:
    rustPlatform.buildRustPackage rec {
      name = "${crate}";

      src = ./.;
      cargoSha256 = "1f2psa1g41pl2j8n60hhik2s2pqdfjhr5capimvajf81kxrnn2ck";

      nativeBuildInputs = [ pkgs.cargo-xbuild ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc.src}/library"
        export CARGO_HOME=$(mktemp -d cargo-home.XXX)
        pushd ${crate}
        cargo xbuild --release --frozen \
          --no-default-features \
          --features=${features}
        popd
      '';

      installPhase = ''
        mkdir -p $out $out/nix-support
        cp target/armv7-none-eabihf/release/${name} $out/${name}.elf
        echo file binary-dist $out/${name}.elf >> $out/nix-support/hydra-build-products
      '';

      doCheck = false;
      dontFixup = true;
    };
in
  {
    zc706-experiments = build-crate "zc706-experiments" "experiments" "target_zc706";
    cora-experiments = build-crate "cora-experiments" "experiments" "target_cora_z7_10";
    redpitaya-experiments = build-crate "redpitaya-experiments" "experiments" "target_redpitaya";
    zc706-fsbl = (import ./nix/fsbl.nix { inherit pkgs; });
    zc706-szl = build-crate "zc706-szl" "szl" "target_zc706";
  }

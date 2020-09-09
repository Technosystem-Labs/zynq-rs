let
  pkgs = import <nixpkgs> { overlays = [ (import ./nix/mozilla-overlay.nix) ]; };
  rustPlatform = (import ./nix/rust-platform.nix { inherit pkgs; });
  build-crate = name: crate: features:
    rustPlatform.buildRustPackage rec {
      name = "${crate}";

      src = ./.;
      cargoSha256 = "1gdxrsn58mabqf8yc3xqkrl1qlmlyvpnzcfj4xafpdr1gdhwhdjg";

      nativeBuildInputs = [ pkgs.cargo-xbuild ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc.src}/src"
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
    zc706-fsbl = (import ./nix/fsbl.nix { inherit pkgs; });
  }

let
  pkgs = import <nixpkgs> { overlays = [ (import ./nix/mozilla-overlay.nix) ]; };
  rustPlatform = (import ./nix/rust-platform.nix { inherit pkgs; });
  cargo-xbuild = (pkgs.cargo-xbuild.overrideAttrs(oa: { patches = oa.patches ++ [ ./xbuild_writable_lockfile.diff ]; } ));
  cargoSha256Experiments = "0ijb3ma7mip48ayc3hilma42rwlz3krb755klmnwwwvxhdhw29w2";
  cargoSha256SZL = "0shz0kzjnmqvwy68km71awn7krn1109nanyckhzvrxy1l1jxg2rd";
  build-crate = name: crate: features: cargoSha256:
    rustPlatform.buildRustPackage rec {
      name = "${crate}";

      src = ./.;
      inherit cargoSha256;

      nativeBuildInputs = [ cargo-xbuild ];
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
    inherit cargo-xbuild;
    zc706-experiments = build-crate "zc706-experiments" "experiments" "target_zc706" cargoSha256Experiments;
    cora-experiments = build-crate "cora-experiments" "experiments" "target_cora_z7_10" cargoSha256Experiments;
    redpitaya-experiments = build-crate "redpitaya-experiments" "experiments" "target_redpitaya" cargoSha256Experiments;
    zc706-fsbl = (import ./nix/fsbl.nix { inherit pkgs; });
    zc706-szl = build-crate "zc706-szl" "szl" "target_zc706" cargoSha256SZL;
    redpitaya-szl = build-crate "redpitaya-szl" "szl" "target_redpitaya" cargoSha256SZL;
  }

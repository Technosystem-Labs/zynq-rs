let
  pkgs = import <nixpkgs> { overlays = [ (import ./nix/mozilla-overlay.nix) ]; };
  rustPlatform = (import ./nix/rust-platform.nix { inherit pkgs; });
  cargo-xbuild = pkgs.callPackage ./nix/cargo-xbuild.nix {};
  cargoSha256Experiments = "0d2v6n6a73vhxm6yq60mvv9pxxxah88kzryb0w32i0ddhfj4rrs7";
  cargoSha256SZL = "0ghj5bbix8417wdismwmxkykvdknikwkvg7sjcjmrywnqk4ka6lg";
  build-crate = name: crate: features: cargoSha256:
    rustPlatform.buildRustPackage rec {
      name = "${crate}";

      src = builtins.filterSource (path: type:
        baseNameOf path != "target"
      ) ./.;
      inherit cargoSha256;

      nativeBuildInputs = [ cargo-xbuild pkgs.llvmPackages_9.clang-unwrapped ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc}/lib/rustlib/src/rust/library"
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

  targetCrates = target: {
    "${target}-experiments" = build-crate "${target}-experiments" "experiments" "target_${target}" cargoSha256Experiments;
    "${target}-szl" = build-crate "${target}-szl" "szl" "target_${target}" cargoSha256SZL;
  };
  targets = ["zc706" "coraz7" "redpitaya" "kasli_soc"];
in
  {
    inherit cargo-xbuild;
    zc706-fsbl = import ./nix/fsbl.nix { inherit pkgs; };
  } // (builtins.foldl' (results: target:
    results // targetCrates target
  ) {} targets)

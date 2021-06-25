let
  pkgs = import <nixpkgs> { overlays = [ (import ./nix/mozilla-overlay.nix) ]; };
  rustPlatform = (import ./nix/rust-platform.nix { inherit pkgs; });
  cargo-xbuild = pkgs.callPackage ./nix/cargo-xbuild.nix {};
  cargoSha256Experiments = "154sdcfs1wazzn1pvm1nqbpary6kq5f3cx5sccz0ss4w8rkj9hr2";
  cargoSha256SZL = "18rbf6vvvz65nc8m0l9y9km97pydhidw4f6yx6yb9vv58yzklsvy";
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
  allTargetCrates = (builtins.foldl' (results: target:
      results // targetCrates target
    ) {} targets);
in
  {
    inherit cargo-xbuild;
    zc706-fsbl = import ./nix/fsbl.nix { inherit pkgs; };
    szl = pkgs.runCommand "szl" {} (builtins.foldl' (commands: target:
      let
        szlResult = builtins.getAttr "${target}-szl" allTargetCrates;
      in
        commands + "ln -s ${szlResult}/szl.elf $out/szl-${target}.elf\n"
    ) "mkdir $out\n" targets);
  } // allTargetCrates

{
  description = "Bare-metal Rust on Zynq-7000";

  inputs.nixpkgs.url = github:NixOS/nixpkgs/nixos-24.11;
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay?ref=snapshot/2024-08-01";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; overlays = [ (import rust-overlay) crosspkgs-overlay ]; };
      
      rust = pkgs.rust-bin.nightly."2024-04-06".default.override {
        extensions = [ "rust-src" ];
        targets = [ ];
      };
      rustPlatform = pkgs.makeRustPlatform {
        rustc = rust // {
          # https://github.com/oxalica/rust-overlay/commit/c48c2d76b68dd9ede0815fec53479375c61af857
          targetPlatforms = pkgs.lib.platforms.all;
          tier1TargetPlatforms = pkgs.lib.platforms.all;
          badTargetPlatforms = [ ];
        };
        cargo = rust;
      };

      crosspkgs-overlay = (self: super: {
        pkgsCross = super.pkgsCross // {
          zynq-baremetal = import super.path {
            system = "x86_64-linux";
            crossSystem = {
              config = "arm-none-eabihf";
              libc = "newlib";
              gcc.cpu = "cortex-a9";
              gcc.fpu = "vfpv3";
            };
          };
        };
      });

      mkbootimage = pkgs.stdenv.mkDerivation {
        pname = "mkbootimage";
        version = "2.3dev";

        src = pkgs.fetchFromGitHub {
          owner = "antmicro";
          repo = "zynq-mkbootimage";
          rev = "872363ce32c249f8278cf107bc6d3bdeb38d849f";
          sha256 = "sha256-5FPyAhUWZDwHbqmp9J2ZXTmjaXPz+dzrJMolaNwADHs=";
        };

        propagatedBuildInputs = [ pkgs.libelf pkgs.pcre ];
        patchPhase =
        ''
          substituteInPlace Makefile --replace "git rev-parse --short HEAD" "echo nix"
        '';
        installPhase =
        ''
          mkdir -p $out/bin
          cp mkbootimage $out/bin
        '';
        hardeningDisable = [ "fortify" ];
      };

      fsbl = { board ? "zc706" }: pkgs.stdenv.mkDerivation {
        name = "${board}-fsbl";
        src = pkgs.fetchFromGitHub {
          owner = "Xilinx";
          repo = "embeddedsw";
          rev = "xilinx_v2022.2";
          sha256 = "sha256-UDz9KK/Hw3qM1BAeKif30rE8Bi6C2uvuZlvyvtJCMfw=";
        };
        nativeBuildInputs = [
          pkgs.pkgsCross.zynq-baremetal.buildPackages.binutils
          pkgs.pkgsCross.zynq-baremetal.buildPackages.gcc
        ];
        patchPhase = ''
          patchShebangs lib/sw_apps/zynq_fsbl/misc/copy_bsp.sh

          for x in lib/sw_apps/zynq_fsbl/src/Makefile lib/sw_apps/zynq_fsbl/misc/copy_bsp.sh lib/bsp/standalone/src/arm/cortexa9/gcc/Makefile; do
            substituteInPlace $x \
              --replace "arm-none-eabi-" "arm-none-eabihf-"
          done
        '';
        buildPhase = ''
          cd lib/sw_apps/zynq_fsbl/src
          make BOARD=${board} "CFLAGS=-DFSBL_DEBUG_INFO -g"
        '';
        installPhase = ''
          mkdir $out
          cp fsbl.elf $out
        '';
        doCheck = false;
        dontFixup = true;
      };

      build-crate = name: crate: features: rustPlatform.buildRustPackage rec {
        name = "${crate}";

        src = builtins.filterSource (path: type:
          baseNameOf path != "target"
        ) ./.;
        cargoLock = { 
          lockFile = ./Cargo.lock;
          outputHashes = {
            "core_io-0.1.0" = "sha256-0HINFWRiJx8pjMgUOL/CS336ih7SENSRh3Kah9LPRrw="; 
            "fatfs-0.3.6" = "sha256-Nz9hCq/1YgSXF8ltJ5ZawV0Hc8WV44KNK0tJdVnNb4U=";
          };
        };

        nativeBuildInputs = [ pkgs.cargo-xbuild pkgs.llvmPackages_14.clang-unwrapped ];
        buildPhase = ''
          export XARGO_RUST_SRC="${rust}/lib/rustlib/src/rust/library"
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
        auditable = false;
      };

      targetCrates = target: {
        "${target}-experiments" = build-crate "${target}-experiments" "experiments" "target_${target}";
        "${target}-szl" = build-crate "${target}-szl" "szl" "target_${target}";
      };
      targets = ["zc706" "coraz7" "redpitaya" "kasli_soc" "ebaz4205"];
      allTargetCrates = (builtins.foldl' (results: target:
        results // targetCrates target
      ) {} targets);
      
      szl = pkgs.runCommand "szl" {} (builtins.foldl' (commands: target:
        let
          szlResult = builtins.getAttr "${target}-szl" allTargetCrates;
        in
          commands + "ln -s ${szlResult}/szl.elf $out/szl-${target}.elf\n"
      ) "mkdir $out\n" targets);
    in rec {
      packages.x86_64-linux = { 
        inherit szl mkbootimage;
        zc706-fsbl = fsbl { board = "zc706"; };
      } // allTargetCrates ;

      hydraJobs = packages.x86_64-linux;

      inherit rust rustPlatform;

      devShell.x86_64-linux = pkgs.mkShell {
        name = "zynq-rs-dev-shell";
        buildInputs = [
          rust
          pkgs.cargo-xbuild
          mkbootimage

          pkgs.openocd pkgs.gdb
          pkgs.openssh pkgs.rsync
          pkgs.llvmPackages_14.clang-unwrapped
          (pkgs.python3.withPackages(ps: [ ps.pyftdi ]))
        ];
      };
    };
}

{
  description = "Bare-metal Rust on Zynq-7000";

  inputs.nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.naersk = {
    url = "github:nix-community/naersk";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    naersk,
  }: let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
      overlays = [(import rust-overlay) crosspkgs-overlay];
    };

    rust = pkgs.rust-bin.nightly."2025-03-28".default.override {
      extensions = ["rust-src"];
      targets = [];
    };
    naerskLib = pkgs.callPackage naersk {
      rustc = rust;
      cargo = rust;
    };

    crosspkgs-overlay = self: super: {
      pkgsCross =
        super.pkgsCross
        // {
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
    };

    mkbootimage = pkgs.stdenv.mkDerivation {
      pname = "mkbootimage";
      version = "2.3dev";

      src = pkgs.fetchFromGitHub {
        owner = "antmicro";
        repo = "zynq-mkbootimage";
        rev = "872363ce32c249f8278cf107bc6d3bdeb38d849f";
        sha256 = "sha256-5FPyAhUWZDwHbqmp9J2ZXTmjaXPz+dzrJMolaNwADHs=";
      };

      propagatedBuildInputs = [pkgs.libelf pkgs.pcre];
      patchPhase = ''
        substituteInPlace Makefile --replace "git rev-parse --short HEAD" "echo nix"
      '';
      installPhase = ''
        mkdir -p $out/bin
        cp mkbootimage $out/bin
      '';
      hardeningDisable = ["fortify"];
    };

    fsbl = {board ? "zc706"}:
      pkgs.stdenv.mkDerivation {
        name = "${board}-fsbl";
        src = pkgs.fetchFromGitHub {
          owner = "Xilinx";
          repo = "embeddedsw";
          rev = "xilinx_v2025.1_update1";
          sha256 = "sha256-XAwhkox1PDyo/UmxP9kjgKsjuoeWgIVhg8X1qFk+Pdo=";
        };
        nativeBuildInputs = [
          pkgs.pkgsCross.zynq-baremetal.buildPackages.binutils
          pkgs.pkgsCross.zynq-baremetal.buildPackages.gcc
        ];

        NIX_CFLAGS_COMPILE = "-DFSBL_DEBUG_INFO -g -no-pie";
        NIX_LDFLAGS = "-no-pie";

        patchPhase = ''
          patchShebangs lib/sw_apps/zynq_fsbl/misc/copy_bsp.sh

          for x in lib/sw_apps/zynq_fsbl/src/Makefile lib/sw_apps/zynq_fsbl/misc/copy_bsp.sh lib/bsp/standalone/src/arm/cortexa9/gcc/Makefile; do
            substituteInPlace $x \
              --replace "arm-none-eabi-" "arm-none-eabihf-"
          done
        '';
        buildPhase = ''
          cd lib/sw_apps/zynq_fsbl/src
          make BOARD=${board}
        '';
        installPhase = ''
          mkdir $out
          cp fsbl.elf $out
        '';
        doCheck = false;
        dontFixup = true;
      };

    build-crate = name: crate: features:
      naerskLib.buildPackage rec {
        name = "${crate}";
        src = ./.;
        additionalCargoLock = "${rust}/lib/rustlib/src/rust/library/Cargo.lock";
        nativeBuildInputs = [pkgs.llvmPackages_20.clang-unwrapped];
        singleStep = true;
        release = true;
        cargoBuildOptions = options:
          options
          ++ [
            "-p ${crate}"
            "--no-default-features"
            "--features=${features}"
          ];
        overrideMain = _: {
          installPhase = ''
            mkdir -p $out $out/nix-support
            cp target/armv7-none-eabihf/release/${name} $out/${name}.elf
            echo file binary-dist $out/${name}.elf >> $out/nix-support/hydra-build-products
          '';
          dontFixup = true;
        };
      };

    targetCrates = target: {
      "${target}-experiments" = build-crate "${target}-experiments" "experiments" "target_${target}";
      "${target}-szl" = build-crate "${target}-szl" "szl" "target_${target}";
    };
    targets = ["zc706" "coraz7" "redpitaya" "kasli_soc" "ebaz4205"];
    allTargetCrates =
      builtins.foldl' (
        results: target:
          results // targetCrates target
      ) {}
      targets;

    szl = pkgs.runCommand "szl" {} (builtins.foldl' (
        commands: target: let
          szlResult = builtins.getAttr "${target}-szl" allTargetCrates;
        in
          commands + "ln -s ${szlResult}/szl.elf $out/szl-${target}.elf\n"
      ) "mkdir $out\n"
      targets);

    fmt-check = pkgs.stdenvNoCC.mkDerivation {
      name = "fmt-check";

      src = ./.;

      nativeBuildInputs = [rust];

      phases = ["unpackPhase" "buildPhase"];

      buildPhase = ''
        cargo fmt -- --check
        touch $out
      '';
    };
  in rec {
    packages.x86_64-linux =
      {
        inherit szl mkbootimage fmt-check;
        zc706-fsbl = fsbl {board = "zc706";};
      }
      // allTargetCrates;

    hydraJobs = packages.x86_64-linux;

    inherit rust naerskLib;

    formatter.x86_64-linux = pkgs.alejandra;

    devShell.x86_64-linux = pkgs.mkShell {
      name = "zynq-rs-dev-shell";
      buildInputs = [
        rust
        pkgs.cargo-xbuild
        mkbootimage

        pkgs.openocd
        pkgs.gdb
        pkgs.openssh
        pkgs.rsync
        pkgs.llvmPackages_20.clang-unwrapped
        (pkgs.python3.withPackages (ps: [ps.pyftdi]))
      ];
    };
  };
}

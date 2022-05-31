{
  description = "Bare-metal Rust on Zynq-7000";

  inputs.nixpkgs.url = github:NixOS/nixpkgs/nixos-22.05;
  inputs.mozilla-overlay = { url = github:mozilla/nixpkgs-mozilla; flake = false; };

  outputs = { self, nixpkgs, mozilla-overlay }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; overlays = [ (import mozilla-overlay) ]; };
      
      rustManifest = pkgs.fetchurl {
        url = "https://static.rust-lang.org/dist/2021-01-29/channel-rust-nightly.toml";
        sha256 = "sha256-EZKgw89AH4vxaJpUHmIMzMW/80wAFQlfcxRoBD9nz0c=";
      };
      rustTargets = [];
      rustChannelOfTargets = _channel: _date: targets:
        (pkgs.lib.rustLib.fromManifestFile rustManifest {
          inherit (pkgs) stdenv lib fetchurl patchelf;
          }).rust.override {
          inherit targets;
          extensions = ["rust-src"];
        };
      rust = rustChannelOfTargets "nightly" null rustTargets;
      rustPlatform = pkgs.recurseIntoAttrs (pkgs.makeRustPlatform {
        rustc = rust;
        cargo = rust;
      });

      gnu-platform = "arm-none-eabi";

      binutils-pkg = { zlib, extraConfigureFlags ? [] }: pkgs.stdenv.mkDerivation rec {
        basename = "binutils";
        version = "2.30";
        name = "${basename}-${gnu-platform}-${version}";
        src = pkgs.fetchurl {
          url = "https://ftp.gnu.org/gnu/binutils/binutils-${version}.tar.bz2";
          sha256 = "028cklfqaab24glva1ks2aqa1zxa6w6xmc8q34zs1sb7h22dxspg";
        };
        configureFlags = [
          "--enable-deterministic-archives"
          "--target=${gnu-platform}"
          "--with-cpu=cortex-a9"
          "--with-fpu=vfpv3"
          "--with-float=hard"
          "--with-mode=thumb"
        ] ++ extraConfigureFlags;
        outputs = [ "out" "info" "man" ];
        depsBuildBuild = [ pkgs.buildPackages.stdenv.cc ];
        buildInputs = [ zlib ];
        enableParallelBuilding = true;
        meta = {
          description = "Tools for manipulating binaries (linker, assembler, etc.)";
          longDescription = ''
            The GNU Binutils are a collection of binary tools.  The main
            ones are `ld' (the GNU linker) and `as' (the GNU assembler).
            They also include the BFD (Binary File Descriptor) library,
            `gprof', `nm', `strip', etc.
          '';
          homepage = http://www.gnu.org/software/binutils/;
          license = pkgs.lib.licenses.gpl3Plus;
          /* Give binutils a lower priority than gcc-wrapper to prevent a
            collision due to the ld/as wrappers/symlinks in the latter. */
          priority = "10";
        };
      };

      gcc-pkg = { gmp, mpfr, libmpc, platform-binutils, extraConfigureFlags ? [] }: pkgs.stdenv.mkDerivation rec {
        basename = "gcc";
        version = "9.1.0";
        name = "${basename}-${gnu-platform}-${version}";
        src = pkgs.fetchurl {
          url = "https://ftp.gnu.org/gnu/gcc/gcc-${version}/gcc-${version}.tar.xz";
          sha256 = "1817nc2bqdc251k0lpc51cimna7v68xjrnvqzvc50q3ax4s6i9kr";
        };
        preConfigure = ''
          mkdir build
          cd build
        '';
        configureScript = "../configure";
        configureFlags = [ 
          "--target=${gnu-platform}"
          "--with-arch=armv7-a"
          "--with-tune=cortex-a9"
          "--with-fpu=vfpv3"
          "--with-float=hard"
          "--disable-libssp"
          "--enable-languages=c"
          "--with-as=${platform-binutils}/bin/${gnu-platform}-as"
          "--with-ld=${platform-binutils}/bin/${gnu-platform}-ld" ] ++ extraConfigureFlags;
        outputs = [ "out" "info" "man" ];
        hardeningDisable = [ "format" "pie" ];
        propagatedBuildInputs = [ gmp mpfr libmpc platform-binutils ];
        enableParallelBuilding = true;
        dontFixup = true;
      };

      newlib-pkg = { platform-binutils, platform-gcc }: pkgs.stdenv.mkDerivation rec {
        pname = "newlib";
        version = "3.1.0";
        src = pkgs.fetchurl {
          url = "ftp://sourceware.org/pub/newlib/newlib-${version}.tar.gz";
          sha256 = "0ahh3n079zjp7d9wynggwrnrs27440aac04340chf1p9476a2kzv";
        };
        nativeBuildInputs = [ platform-binutils platform-gcc ];
        configureFlags = [
          "--target=${gnu-platform}"

          "--with-cpu=cortex-a9"
          "--with-fpu=vfpv3"
          "--with-float=hard"
          "--with-mode=thumb"
          "--enable-interwork"
          "--disable-multilib"

          "--disable-newlib-supplied-syscalls"
          "--with-gnu-ld"
          "--with-gnu-as"
          "--disable-newlib-io-float"
          "--disable-werror"
        ];
        dontFixup = true;
      };
      gnutoolchain = rec {
        binutils-bootstrap = pkgs.callPackage binutils-pkg { };
        gcc-bootstrap = pkgs.callPackage gcc-pkg {
          platform-binutils = binutils-bootstrap;
          extraConfigureFlags = [ "--disable-libgcc" ];
        };
        newlib = pkgs.callPackage newlib-pkg {
          platform-binutils = binutils-bootstrap;
          platform-gcc = gcc-bootstrap;
        };
        binutils = pkgs.callPackage binutils-pkg {
          extraConfigureFlags = [ "--with-lib-path=${newlib}/arm-none-eabi/lib" ];
        };
        gcc = pkgs.callPackage gcc-pkg {
          platform-binutils = binutils;
          extraConfigureFlags = [ "--enable-newlib" "--with-headers=${newlib}/arm-none-eabi/include" ];
        };
      };

      cargo-xbuild = rustPlatform.buildRustPackage rec {
        pname = "cargo-xbuild";
        version = "0.6.5";

        src = pkgs.fetchFromGitHub {
          owner = "rust-osdev";
          repo = pname;
          rev = "v${version}";
          sha256 = "18djvygq9v8rmfchvi2hfj0i6fhn36m716vqndqnj56fiqviwxvf";
        };
        cargoSha256 = "13sj9j9kl6js75h9xq0yidxy63vixxm9q3f8jil6ymarml5wkhx8";

        meta = with pkgs.lib; {
          description = "Automatically cross-compiles the sysroot crates core, compiler_builtins, and alloc";
          homepage = "https://github.com/rust-osdev/cargo-xbuild";
          license = with licenses; [ mit asl20 ];
          maintainers = with maintainers; [ johntitor xrelkd ];
        };
      };

      mkbootimage = pkgs.stdenv.mkDerivation {
        pname = "mkbootimage";
        version = "2.2";

        src = pkgs.fetchFromGitHub {
          owner = "antmicro";
          repo = "zynq-mkbootimage";
          rev = "4ee42d782a9ba65725ed165a4916853224a8edf7";
          sha256 = "1k1mbsngqadqihzjgvwvsrkvryxy5ladpxd9yh9iqn2s7fxqwqa9";
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
      };

      fsbl = { board ? "zc706" }: pkgs.stdenv.mkDerivation {
        name = "${board}-fsbl";
        src = pkgs.fetchFromGitHub {
          owner = "Xilinx";
          repo = "embeddedsw";
          rev = "65c849ed46c88c67457e1fc742744f96db968ff1";
          sha256 = "1rvl06ha40dzd6s9aa4sylmksh4xb9dqaxq462lffv1fdk342pda";
        };
        patches = [ ./fsbl.patch ];
        nativeBuildInputs = [
          pkgs.gnumake
          gnutoolchain.binutils
          gnutoolchain.gcc
        ];
        patchPhase = ''
          patch -p1 -i ${./fsbl.patch}
          patchShebangs lib/sw_apps/zynq_fsbl/misc/copy_bsp.sh
          echo 'SEARCH_DIR("${gnutoolchain.newlib}/arm-none-eabi/lib");' >> lib/sw_apps/zynq_fsbl/src/lscript.ld
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
        cargoLock = { lockFile = ./Cargo.lock; };

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
        "${target}-experiments" = build-crate "${target}-experiments" "experiments" "target_${target}";
        "${target}-szl" = build-crate "${target}-szl" "szl" "target_${target}";
      };
      targets = ["zc706" "coraz7" "redpitaya" "kasli_soc"];
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
        inherit cargo-xbuild szl mkbootimage;
        zc706-fsbl = fsbl { board = "zc706"; };
      } // allTargetCrates ;

      hydraJobs = packages.x86_64-linux;

      inherit rustPlatform;

      devShell.x86_64-linux = pkgs.mkShell {
        name = "zynq-rs-dev-shell";
        buildInputs = with pkgs; [
          rustPlatform.rust.rustc
          rustPlatform.rust.cargo
          cacert
          cargo-xbuild

          openocd gdb
          openssh rsync
          llvmPackages_9.clang-unwrapped
          (python3.withPackages(ps: [ ps.pyftdi ]))
          mkbootimage ];
        };
    };
}
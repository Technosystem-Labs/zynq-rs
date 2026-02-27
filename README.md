# Bare-metal Rust on Zynq-7000

Supported features:

* Clocking setup
* UART
* SDRAM setup
* Ethernet with smoltcp and async-await on TCP sockets
* SD card
* PL programming and startup
* Pure Rust SZL first-stage bootloader with SD boot
* SZL UART SD file service with subdirectory support (list/upload/download/delete/mkdir/rmdir/reboot/format)
* Control of second CPU core and message passing, with async-await support


Supported boards:
 * Kasli-SoC
 * ZC706
 * Red Pitaya
 * Cora Z7-10 (seems to also run on Cora Z7-07S, including dual-core support)

## Build

Zynq-rs is packaged using the [Nix](https://nixos.org) Flakes system. Install Nix 2.4+ and enable flakes by adding ``experimental-features = nix-command flakes`` to ``nix.conf`` (e.g. ``~/.config/nix/nix.conf``). 

You can build SZL or experiments crate for the platform of your choice by using ``nix build`` command, e.g.

```shell
nix build .#coraz7-experiments
```

Alternatively, you can still use ``cargo build`` within a ``nix develop`` shell.

```shell
nix develop
cargo build --release -p experiments
```

Currently the ELF output is placed at `target/armv7-none-eabihf/release/experiments`, or `result/experiments.elf` for Nix Flakes build.

## Debug

### Running on the ZC706

```shell
nix develop
cargo build --release -p experiments
cd openocd
openocd -f zc706.cfg
```

### Running on the Cora Z7-10

```shell
nix develop
cargo build --release -p experiments --no-default-features --features=target_coraz7
cd openocd
openocd -f cora-z7-10.cfg
```

### Loading a bitstream into volatile memory

```shell
openocd -f zc706.cfg -c "pld load 0 blinker_migen.bit; exit"
```

## SZL SD File Service

`szl-sdfs` boots into an SD-backed file service over UART.
After boot, it prints `SZL-SD READY v1; switching to binary protocol` and accepts framed binary commands.

You can run the client directly from the flake with:

```shell
# load ELF image into target via JTAG
nix run .#kasli_soc-szl-sdfs-cli -- load

# list files and directories in SD root
nix run .#kasli_soc-szl-sdfs-cli -- list

# list contents of a subdirectory
nix run .#kasli_soc-szl-sdfs-cli -- list ARTIQ

# upload local file (supports subdirectory paths)
nix run .#kasli_soc-szl-sdfs-cli -- upload BOOT.BIN BOOT.BIN
nix run .#kasli_soc-szl-sdfs-cli -- upload local.bin ARTIQ/CONFIG.BIN

# download remote file to local path
nix run .#kasli_soc-szl-sdfs-cli -- download BOOT.BIN BOOT.BIN
nix run .#kasli_soc-szl-sdfs-cli -- download ARTIQ/CONFIG.BIN config.bin

# delete a file
nix run .#kasli_soc-szl-sdfs-cli -- delete BOOT.BIN

# create a directory (creates intermediate directories as needed)
nix run .#kasli_soc-szl-sdfs-cli -- mkdir ARTIQ/CONFIG

# remove an empty directory
nix run .#kasli_soc-szl-sdfs-cli -- rmdir ARTIQ/CONFIG

# reboot target
nix run .#kasli_soc-szl-sdfs-cli -- reboot

# format the SD card (writes new MBR + FAT32 partition, erases all data)
nix run .#kasli_soc-szl-sdfs-cli -- format --yes
```

Notes:
* UART baud rate for this service is fixed at `1500000`.
* All filenames use 8.3 format (base 1–8 chars, extension 0–3 chars; allowed chars: `A-Z`, `0-9`, `_`, `-`); lowercase is accepted and converted to uppercase automatically.
* Paths use `/` as separator; leading/trailing slashes are not allowed.
* `format` rewrites the MBR and creates a fresh FAT32 partition starting at LBA 1. All existing data is lost.
* If you get a `error: access to absolute path` error, try with `--impure` or upgrade your Nix (see [PR #12045](https://github.com/NixOS/nix/pull/12045))
* When ARTIQ-Zynq is already present on the target SD card, the `load` command (OpenOCD JTAG load + resume) often works only if run shortly after power cycling the device.
  Workaround: power cycle the board, then run `load` shortly after the power is restored.

## License

Copyright (C) 2019-2025 M-Labs Limited.
Copyright (C) 2026 TechnoSystem (SD File Service szl modification).
Released under the GNU LGPL v3. See the LICENSE file for details.

# Bare-metal Rust on Zynq-7000

Supported features:

* Clocking setup
* UART
* SDRAM setup
* Ethernet with smoltcp and async-await on TCP sockets
* SD card
* PL programming and startup
* Pure Rust SZL first-stage bootloader, with SD boot and netboot
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

Alternatively, you can still use ``cargo xbuild`` within ``nix develop`` shell.

```shell
nix develop
cargo xbuild --release -p experiments
```

Currently the ELF output is placed at `target/armv7-none-eabihf/release/experiments`, or `result/experiments.elf` for Nix Flakes build.

## Debug

### Running on the ZC706

```shell
nix develop
cargo xbuild --release -p experiments
cd openocd
openocd -f zc706.cfg
```

### Running on the Cora Z7-10

```shell
nix develop
cargo xbuild --release -p experiments --no-default-features --features=target_coraz7
cd openocd
openocd -f cora-z7-10.cfg
```

### Loading a bitstream into volatile memory

```shell
openocd -f zc706.cfg -c "pld load 0 blinker_migen.bit; exit"
```

## License

Copyright (C) 2019-2021 M-Labs Limited.
Released under the GNU LGPL v3. See the LICENSE file for details.

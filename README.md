# Build

```shell
nix-shell --command "cargo xbuild --release -p experiments"
```

Currently the ELF output is placed at `target/armv7-none-eabihf/release/experiments`

# Debug

## Running on the ZC706

```shell
nix-shell --command "cargo xbuild --release -p experiments"
cd openocd
openocd -f zc706.cfg
```

## Running on the Cora Z7-10

```shell
nix-shell --command "cd experiments && cargo xbuild --release --no-default-features --features=target_cora_z7_10"
cd openocd
openocd -f cora-z7-10.cfg
```

## Loading a bitstream into volatile memory

```shell
openocd -f zc706.cfg -c "pld load 0 blinker_migen.bit; exit"
```

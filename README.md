# Build

```shell
nix-shell --command "cargo build --release"
```

# Debug

## Using the Xilinx toolchain

Tested with the ZC706 board.

Run the Xilinx Microprocessor Debugger:
```shell
/opt/Xilinx/14.7/ISE_DS/EDK/bin/lin64/xmd
```

Connect to target (given it is connected and you have permissions):
```tcl
connect arm hw
```

Leave xmd running.

Start the Xilinx version of the GNU debugger with your latest build:
```shell
/opt/Xilinx/14.7/ISE_DS/EDK/gnu/arm/lin/bin/arm-xilinx-linux-gnueabi-gdb zc706
```

Connect the debugger to xmd over TCP on localhost:
```gdb
target remote :1234
```

Proceed using gdb with `load`, `c`

## Using OpenOCD

### Resources for the ZC706

https://devel.rtems.org/wiki/Debugging/OpenOCD/Xilinx_Zynq
https://github.com/nathanrossi/meta-random/tree/master/openocd-zynq

### Running on the Cora Z7-10

```shell
nix-shell --command "cargo build --release --no-default-features --features=target_cora_z7_10"
openocd -f cora-z7-10.cfg
```

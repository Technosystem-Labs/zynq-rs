# Debugging

## Using the Xilinx toolchain

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

## Using OpenOCD: not working


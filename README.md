# Build

```shell
nix-shell --command "cargo xbuild --release"
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

### Running on the ZC706

```shell
nix-shell --command "cargo xbuild --release"
cd openocd
openocd -f zc706.cfg
```

### Running on the Cora Z7-10

```shell
nix-shell --command "cargo xbuild --release --no-default-features --features=target_cora_z7_10"
cd openocd
openocd -f cora-z7-10.cfg
```

### Development Process

Clone this repo onto your development/build machine and the raspberry pi that controls the Xilinx 7000 board

On the dev machine, the below script builds zc706 and secure copies it to the target pi (in your pi $HOME directory)
```shell
cd ~/zc706
./build.sh $your_user/ssh_id
```

On the pi, we need an information rich environment that includes a relatively reliable `gdb` experience (that includes `ctrl-p` and `ctrl-n` command history that persists across `cgdb` executions), run:
```shell
ssh pi4
cd zc706
./tmux.sh
```

Time to run your code with:
```shell
zynq-connect
zynq-restart
c
```
or, for a more succinct experience, (identical to above)
```shell
dc
dr
c
```

After every build on your dev machine, simply run:
```shell
dr
c
```
Sometimes you might need to type `load` after `dr`.

Note, to exit `picocom` hit `ctrl-a x`


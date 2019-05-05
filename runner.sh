#!/usr/bin/env bash

set -e -m

ELF=$1
IMAGE=$ELF.bin
arm-none-eabihf-objcopy -O binary $ELF $IMAGE
qemu-system-arm -M xilinx-zynq-a9 -s -kernel $IMAGE -chardev file,id=uart0,path=/tmp/qemu.serial &
sleep 1
gdb -x qemu.gdb $ELF
kill -KILL %1

#!/usr/bin/env bash

set -e

target_host="rpi-4.m-labs.hk"

while getopts "h:i" opt; do
    case "$opt" in
    \?) exit 0
        ;;
    h)  target_host=$OPTARG
        ;;
    esac
done

target_folder=/tmp/zynq-\$USER

ssh $target_host "mkdir -p $target_folder"
rsync openocd/* $target_host:$target_folder
rsync target/armv7-none-eabihf/release/experiments $target_host:$target_folder/experiments.elf
ssh $target_host "cd $target_folder; openocd -f zc706.cfg -c 'load_image experiments.elf; resume 0; exit'"

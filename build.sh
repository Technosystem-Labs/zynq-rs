nix-shell --command "cargo xbuild --release -p experiments" && scp -C target/armv7-none-eabihf/release/experiments $1@rpi-4.m-labs.hk:/home/$1/zc706/zc706.elf

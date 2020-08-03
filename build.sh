nix-shell --command "cargo xbuild --release -p experiments" && scp -P 2204 -C target/armv7-none-eabihf/release/zc706-experiments $1@nixbld.m-labs.hk:/home/$1/zc706/zc706.elf

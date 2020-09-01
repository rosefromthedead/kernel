##!/bin/sh

set -e

RUSTFLAGS=-Clink-arg=-Taarch64.ld cargo xbuild --target aarch64-unknown-none --release

aarch64-linux-gnu-objcopy -S -O binary target/aarch64-unknown-none/debug/kernel build/kernel.bin
./mkubootimage -u -f arm64 -A arm64 -O linux -T kernel -C none -a 0x200000 build/kernel.bin build/kernel.ub

qemu-system-aarch64 -M virt -cpu cortex-a53 -m 1g -nographic -kernel build/kernel.ub $@

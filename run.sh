#!/usr/bin/env bash

set -e

mkdir -p build

cd kernel
./build.sh
cd ..

cd init
cargo build --target aarch64-unknown-none
cd ..

qemu-system-aarch64 -M virt -cpu cortex-a53 -m 1g -nographic -kernel build/kernel.ub -initrd target/aarch64-unknown-none/debug/init $@

#!/usr/bin/env bash

set -e

cd kernel
./build.sh
cd ..

cd init
cargo build --target aarch64-unknown-none -Zbuild-std=core,alloc
cd ..

qemu-system-aarch64 -M virt -cpu cortex-a53 -m 1g -nographic -kernel kernel/build/kernel.ub -initrd init/target/aarch64-unknown-none/debug/init $@

#!/usr/bin/env bash

set -e

toolchain_prefix=aarch64-unknown-linux-gnu-
if [ $(uname -m) = aarch64 ]; then
    toolchain_prefix=
fi

# use gnu ld - default lld gives some weird relocation error and i don't want to deal with it
RUSTFLAGS="-Clinker=${toolchain_prefix}gcc -Clink-arg=-Tkernel/aarch64.ld -Clink-arg=-nostdlib" \
    cargo build --target aarch64-unknown-none --features build-asm -Zbuild-std=core,alloc

${toolchain_prefix}objcopy -S -O binary ../target/aarch64-unknown-none/debug/kernel ../build/kernel.bin
mkarm64image --overwrite --entry-point=0x200000 ../build/kernel.bin ../build/kernel.ub

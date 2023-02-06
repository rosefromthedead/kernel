#!/usr/bin/env bash

set -e

# use gnu ld - default lld gives some weird relocation error and i don't want to deal with it
RUSTFLAGS="-Clinker=aarch64-unknown-linux-gnu-gcc -Clink-arg=-Tkernel/aarch64.ld" \
    cargo build --target aarch64-unknown-none --features build-asm

aarch64-unknown-linux-gnu-objcopy -S -O binary ../target/aarch64-unknown-none/debug/kernel ../build/kernel.bin
mkarm64image --overwrite --entry-point=0x200000 ../build/kernel.bin ../build/kernel.ub

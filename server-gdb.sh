#!/bin/bash

cargo build --release

rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/six_os -O binary target/riscv64gc-unknown-none-elf/release/six_os.bin

qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=target/riscv64gc-unknown-none-elf/release/six_os.bin,addr=0x80200000 \
	-s -S

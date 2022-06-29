#!/bin/bash

riscv64-unknown-elf-gdb \
	-ex 'file target/riscv64gc-unknown-none-elf/release/six_os' \
	-ex 'set arch riscv:rv64' \
	-ex 'target remote 127.0.0.1:1234'

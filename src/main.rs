#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;
mod config;
mod kernel;
mod memory;
mod rust_need;
mod sbi;
mod syscall;
mod loader;

use kernel::clear_bss;

use core::arch::global_asm;
global_asm!(include_str!("kernel/entry.asm"));
global_asm!(include_str!("kernel/link_app.S"));

#[no_mangle]
pub fn rust_init() {
    clear_bss();
    memory::memory_init();
}

#[no_mangle]
pub fn rust_main() -> ! {
    println!("Hello, world!");
    panic!("Hit the Bottom!");
}

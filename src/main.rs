#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;
mod config;
mod kernel;
mod ktype;
mod memory;
mod rust_need;
mod sbi;

use kernel::clear_bss;
use memory::heap_manager::init_heap;

use core::arch::global_asm;

use crate::config::MEMORY_END;
global_asm!(include_str!("kernel/entry.asm"));

#[no_mangle]
pub fn rust_init() {
    clear_bss();
    //init_heap();

    //use crate::memory::heap_manager::heap_test;
    //heap_test();

    //use crate::memory::mem_manager::mem_init;
    //mem_init();

    memory::memory_init();

    //println!("Hello, world!");
    //panic!("Hit the Bottom!");
}

#[no_mangle]
pub fn rust_main() -> ! {
    println!("Hello, world!");
    panic!("Hit the Bottom!");
}

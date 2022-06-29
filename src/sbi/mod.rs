#![allow(unused)]

use crate::ktype::Kusize;

const SBI_SET_TIMER: Kusize = 0;
const SBI_CONSOLE_PUTCHAR: Kusize = 1;
const SBI_CONSOLE_GETCHAR: Kusize = 2;
const SBI_CLEAR_IPI: Kusize = 3;
const SBI_SEND_IPI: Kusize = 4;
const SBI_REMOTE_FENCE_I: Kusize = 5;
const SBI_REMOTE_SFENCE_VMA: Kusize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: Kusize = 7;
const SBI_SHUTDOWN: Kusize = 8;

use core::arch::asm;

#[inline(always)]
fn sbi_call(which: Kusize, arg0: Kusize, arg1: Kusize, arg2: Kusize) -> Kusize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    }
    ret
}

pub fn console_putchar(c: Kusize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}

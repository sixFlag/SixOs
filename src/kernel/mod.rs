use crate::ktype::Kusize;

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as Kusize..ebss as Kusize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) })
}

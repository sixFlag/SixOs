use crate::ktype::Kusize;
pub const MEMORY_END: Kusize = 0x80800000;
pub const MEMORY_START: Kusize = 0x80200000;
pub const KERNEL_STACK_SIZE: Kusize = 8 * 4096;
pub const PAGESIZE: Kusize = 4096;

use crate::config::{PAGESIZE, MEMORY_END, MEMORY_START, KERNEL_STACK_SIZE};
use crate::ktype::Kusize;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::sync::Arc;
use bitflags::*;
use lazy_static::*;
use core::arch::asm;
use riscv::register::satp;


//----------------------------------------------------------------
use core::cell::{RefCell, RefMut};
pub struct UPSafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}


lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<PageTable>> = Arc::new(unsafe {
        UPSafeCell::new(PageTable::init_kernel_space()
    )});
}

//----------------------------------------------------------------

extern "C" {
    fn skernel();
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn ekernel();

    fn kernel_stack_bottom();
    fn kernel_stack_top();
}

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[repr(C, align(4096))]
#[derive(Debug)]
struct Page {
    data: [u8; PAGESIZE as usize],
}

pub struct VirtualAddress {
    address: Kusize,
}

pub struct PhysicalAddress {
    address: Kusize,
}

impl VirtualAddress {
    pub fn indexes(&self) -> [Kusize; 3] {
        let mut vpn = self.address >> 12;
        let mut idx: [Kusize; 3] = [0; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

pub struct PageTable {
    root_box: Box<Page>,
    page_table_pages: Vec<Box<Page>>,
    data_pages: BTreeMap<VirtualAddress, Box<Page>>,
}

pub struct PageTableEntry {
    pub bits: Kusize,
}

impl PageTableEntry {
    pub fn new(physical_address: PhysicalAddress, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: (physical_address.address / PAGESIZE) << 10 | flags.bits as Kusize,
        }
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

impl PageTable {
    fn init() -> Self {
        PageTable {
            root_box: Box::new(Page { data: [0; 4096] }),
            page_table_pages: Vec::new(),
            data_pages: BTreeMap::new(),
        }
    }

    fn find_pte_create(&mut self, virtual_address: VirtualAddress) -> Option<&mut PageTableEntry> {
        let idxs = virtual_address.indexes();
        let mut result: Option<&mut PageTableEntry> = None;

        let mut physical_address = &(self.root_box.data[0]) as *const _ as Kusize;

        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut unsafe {
                core::slice::from_raw_parts_mut(physical_address as *mut PageTableEntry, 512)
            }[(*idx) as usize];

            if i == 2 {
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                let temp = Box::new(Page { data: [0; 4096] });
                *pte = PageTableEntry::new(PhysicalAddress{ address: &(temp.data[0]) as *const _ as Kusize }, PTEFlags::V);

                self.page_table_pages.push(temp);
            }

            physical_address = ((*pte).bits >> 10) << 12;
        }

        result
    }

    fn find_pte(&self, virtual_address: VirtualAddress) -> Option<&mut PageTableEntry> {
        let idxs = virtual_address.indexes();
        let mut result: Option<&mut PageTableEntry> = None;

        let mut physical_address = &(self.root_box.data[0]) as *const _ as Kusize;

        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut unsafe {
                core::slice::from_raw_parts_mut(physical_address as *mut PageTableEntry, 512)
            }[(*idx) as usize];

            if i == 2 {
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                return None;
            }

            physical_address = ((*pte).bits >> 10) << 12;
        }

        result
    }


    fn map_kernel(&mut self) {

        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
        println!(".ekernel [{:#x}, {:#x})", ekernel as usize, MEMORY_END as usize);
        println!(".kernel_stack_bottom [{:#x}, {:#x})", kernel_stack_bottom as usize, kernel_stack_top as usize);


        for i in (stext as Kusize..etext as Kusize).step_by(PAGESIZE as usize) {
            *(self.find_pte_create(VirtualAddress{ address: i}).unwrap()) = 
                PageTableEntry::new(PhysicalAddress{ address: i }, PTEFlags::V | PTEFlags::R | PTEFlags::X );
        }

        for i in (srodata as Kusize..erodata as Kusize).step_by(PAGESIZE as usize) {
            *(self.find_pte_create(VirtualAddress{ address: i}).unwrap()) = 
                PageTableEntry::new(PhysicalAddress{ address: i }, PTEFlags::V | PTEFlags::R );
        }      
        
        for i in (sdata as Kusize..edata as Kusize).step_by(PAGESIZE as usize) {
            *(self.find_pte_create(VirtualAddress{ address: i}).unwrap()) = 
                PageTableEntry::new(PhysicalAddress{ address: i }, PTEFlags::V | PTEFlags::R | PTEFlags::W );
        }        

        for i in (sbss as Kusize..ebss as Kusize).step_by(PAGESIZE as usize) {
            *(self.find_pte_create(VirtualAddress{ address: i}).unwrap()) = 
                PageTableEntry::new(PhysicalAddress{ address: i }, PTEFlags::V | PTEFlags::R | PTEFlags::W );
        }          

        for i in (ekernel as Kusize..MEMORY_END as Kusize).step_by(PAGESIZE as usize) {
            *(self.find_pte_create(VirtualAddress{ address: i}).unwrap()) = 
                PageTableEntry::new(PhysicalAddress{ address: i }, PTEFlags::V | PTEFlags::R | PTEFlags::W );
        }  
        
        for i in (( (MEMORY_START - KERNEL_STACK_SIZE) as Kusize)..MEMORY_START as Kusize).step_by(PAGESIZE as usize) {
            *(self.find_pte_create(VirtualAddress{ address: i}).unwrap()) = 
                PageTableEntry::new(PhysicalAddress{ address: (i + KERNEL_STACK_SIZE + ( kernel_stack_bottom as Kusize - MEMORY_START ) ) }, PTEFlags::V | PTEFlags::R | PTEFlags::W );
        }

        println!("mapped kernel done!");


        //test
        // println!("test: {:#x}", self.find_pte(VirtualAddress {address: stext as Kusize}).unwrap().bits >> 10 << 12  );
        // println!("test: {:#x}", self.find_pte(VirtualAddress {address: srodata as Kusize}).unwrap().bits >> 10 << 12  );
        // println!("test: {:#x}", self.find_pte(VirtualAddress {address: sdata as Kusize}).unwrap().bits >> 10 << 12  );
        // println!("test: {:#x}", self.find_pte(VirtualAddress {address: sbss as Kusize}).unwrap().bits >> 10 << 12  );
        // println!("test: {:#x}", self.find_pte(VirtualAddress {address: ekernel as Kusize}).unwrap().bits >> 10 << 12  );

    }

    pub fn init_kernel_space() -> PageTable {
        let mut kernel_space = PageTable::init();
        kernel_space.map_kernel();
        kernel_space
    }

    pub fn token(&self) -> Kusize {
        (8 as Kusize) << 60 | ((&(self.root_box.data[0]) as *const _ as Kusize) >> 12 )
    }    

    pub fn activate(&self) {
        let satp = self.token();
        unsafe {
            satp::write(satp as usize);
            asm!("sfence.vma");
        }
    }

}

pub fn init_mem() {
    KERNEL_SPACE.exclusive_access().activate();
}

// extern "C" {
//     fn ekernel();
// }
// println!("This is a test 0x{:X}",  ekernel as usize );
// println!("This is a test 0x{:X}",  &a as *const _ as usize );
// println!("This is a test 0x{:X}",  &(a.data[1]) as *const _ as usize );
// println!("This is a test 0x{:X}",  &(a.data[2]) as *const _ as usize );
// println!("This is a test 0x{:X}",  &(a.data[3]) as *const _ as usize );
// println!("This is a test 0x{:X}",  &(a.data[4]) as *const _ as usize );

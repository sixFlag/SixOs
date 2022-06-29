use crate::config::MEMORY_END;
use crate::ktype::Kusize;
use buddy_system_allocator::LockedHeap;

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    extern "C" {
        fn ekernel();
    }

    unsafe {
        HEAP_ALLOCATOR.lock().init(
            (ekernel as Kusize).try_into().unwrap(),
            (MEMORY_END - (ekernel as Kusize)).try_into().unwrap(),
        );
    }
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn ekernel();
    }
    let bss_range = ekernel as Kusize..MEMORY_END as Kusize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as Kusize)));
    drop(a);
    let mut v: Vec<Kusize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!((*val) as usize, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as Kusize)));
    drop(v);
    println!("heap_test passed!");
}

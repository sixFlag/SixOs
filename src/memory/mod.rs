pub mod heap_manager;
pub mod mem_manager;

pub fn memory_init() {
    heap_manager::init_heap();
    mem_manager::init_mem();
}

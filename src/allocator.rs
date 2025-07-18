use crate::println;
use memory_addr::{
    PAGE_SIZE_4K, PageIter, PhysAddr, VirtAddr, align_down_4k, align_up_4k,
};
use page_table_multiarch::riscv::Sv39PageTable;
use page_table_multiarch::{MappingFlags, PageSize, PagingResult};
use crate::allocator::buddy::BuddyAllocator;
use crate::page::FrameAllocator;

pub mod buddy;
pub mod fixed_size_block;

/// A wrapper around spin::Mutex to permit trait implementation.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

#[global_allocator]
static ALLOCATOR: Locked<BuddyAllocator> = Locked::new(BuddyAllocator::new());

pub const HEAP_SIZE: usize = 1024 * 1024;
pub const HEAP_START: usize = 0x80700000;
// pub const STACK_START: usize = RAM_START + RAM_SIZE;
// pub const STACK_SIZE: usize = 1024;

pub fn init_heap() -> PagingResult {
    println!("Initializing kernel heap");

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    println!("Finished initializing kernel heap");

    Ok(())
}

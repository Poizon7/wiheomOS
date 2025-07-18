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

unsafe extern "C" {
    static __sheap: u8;
    static __eheap: u8;
}

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

// pub const STACK_START: usize = RAM_START + RAM_SIZE;
// pub const STACK_SIZE: usize = 1024;

pub fn init_heap() -> PagingResult {
    println!("Initializing kernel heap");
    let heap_start = unsafe { &__sheap as *const u8 as usize };
    let heap_end = unsafe { &__eheap as *const u8 as usize };
    let heap_size = heap_end - heap_start;

    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }

    println!("Finished initializing kernel heap");

    Ok(())
}

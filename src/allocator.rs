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

pub const HEAP_SIZE: usize = 1024;
// pub const STACK_START: usize = RAM_START + RAM_SIZE;
// pub const STACK_SIZE: usize = 1024;

pub fn init_heap(page_table: &mut Sv39PageTable<FrameAllocator>) -> PagingResult {
    println!("Initializing kernel heap");
    let heap_start = riscv_rt::heap_start() as usize;
    let heap_end = heap_start + HEAP_SIZE - 1usize;
    let heap_start_page = PhysAddr::from(align_down_4k(heap_start));
    let heap_end_page = PhysAddr::from(align_up_4k(heap_end));
    let page_range =
        PageIter::<PAGE_SIZE_4K, PhysAddr>::new(heap_start_page, heap_end_page).unwrap();

    for page in page_range {
        let frame = VirtAddr::from(page.as_usize());
        let flags = MappingFlags::WRITE | MappingFlags::READ;
        let _ = page_table.map(frame, page, PageSize::Size4K, flags)?;
    }

    unsafe {
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }

    println!("Finished initializing kernel heap");

    Ok(())
}

use memory_addr::PhysAddr;
use page_table_multiarch::{PagingHandler, riscv::Sv48PageTable};
use memory_addr::{PageIter, VirtAddr, PAGE_SIZE_4K};
use riscv::register::satp::{self, Mode, Satp};
use spin::Mutex;

use crate::println;


pub struct FrameAllocator {
    start: PhysAddr,
    end: PhysAddr,
    next: usize,
}

pub const RAM_START: usize = 0x80600000;
pub const RAM_SIZE: usize = 2 * 1024 * 1024;


static mut FRAME_ALLOCATOR: Mutex<Option<FrameAllocator>> = Mutex::new(None);

impl FrameAllocator {
    pub unsafe fn init(start: usize, size: usize) {
        #[allow(static_mut_refs)]
        let mut allocator = unsafe { FRAME_ALLOCATOR.lock() };
        *allocator = Some(Self {
            start: PhysAddr::from_usize(start),
            end: PhysAddr::from_usize(start + size),
            next: 0,
        });
    }

    fn usable_frames(&self) -> PageIter<PAGE_SIZE_4K, PhysAddr> {
        PageIter::<PAGE_SIZE_4K, PhysAddr>::new(self.start, self.end).unwrap()
    }
}

impl PagingHandler for FrameAllocator {
    fn alloc_frame() -> Option<PhysAddr> {
        #[allow(static_mut_refs)]
        let mut allocator = unsafe { FRAME_ALLOCATOR.lock() };
        let allocator = allocator.as_mut().unwrap();
        let frame = allocator.usable_frames().nth(allocator.next);
        allocator.next += 1;
        frame
    }

    fn dealloc_frame(_paddr: PhysAddr) {}

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        VirtAddr::from(paddr.as_usize())
    }
}

pub unsafe fn init_frame_allocator() {
    println!("Initializing frame allocator");
    unsafe { FrameAllocator::init(RAM_START, RAM_SIZE) };
    println!("Finished initializing frame allocator");
}

pub unsafe fn init_page_table() -> Sv48PageTable<FrameAllocator> {
    println!("Initializing page table");
    let page_table = Sv48PageTable::<FrameAllocator>::try_new().unwrap();
    let mut reg = Satp::from_bits(0);
    reg.set_mode(Mode::Sv48);
    reg.set_ppn(page_table.root_paddr().as_usize());
    // println!("Turn on MMU");
    // unsafe { satp::write(reg) };
    println!("Finished initializing page table");
    page_table
}

use memory_addr::{PAGE_SIZE_4K, PageIter, VirtAddr};
use memory_addr::{PhysAddr, align_down_4k, align_up_4k};
use page_table_multiarch::{MappingFlags, PageSize, PagingError};
use page_table_multiarch::{PagingHandler, riscv::Sv39PageTable};
use riscv::register::satp::{self, Mode, Satp};
use spin::Mutex;

use crate::println;

unsafe extern "C" {
    static __stext: u8;
    static __etext: u8;

    static __srodata: u8;
    static __erodata: u8;

    static __sdata: u8;
    static __edata: u8;

    static __sbss: u8;
    static __ebss: u8;

    static _stack_start: u8;
}

pub fn addr(sym: *const u8) -> usize {
    sym as usize
}

pub fn map_section(
    page_table: &mut Sv39PageTable<FrameAllocator>,
    start: *const u8,
    end: *const u8,
    flags: MappingFlags,
) -> Result<(), PagingError> {
    let start = align_down_4k(addr(start));
    let end = align_up_4k(addr(end));
    println!(
        "Mapped section from {:#x} to {:#x} with flags {:?}",
        start, end, flags
    );
    let _ = page_table.map_region(
        VirtAddr::from(start),
        |vaddr| PhysAddr::from(vaddr.as_usize()),
        end - start,
        flags,
        false,
        true,
    )?;
    Ok(())
}

pub fn map_section_size(
    page_table: &mut Sv39PageTable<FrameAllocator>,
    start: *const u8,
    size: usize,
    flags: MappingFlags,
) -> Result<(), PagingError> {
    let start = align_down_4k(addr(start));
    let end = align_up_4k(start + size);
    println!(
        "Mapped section from {:#x} to {:#x} with flags {:?}",
        start, end, flags
    );
    let _ = page_table.map_region(
        VirtAddr::from(start),
        |vaddr| PhysAddr::from(vaddr.as_usize()),
        end - start,
        flags,
        false,
        true,
    )?;
    Ok(())
}

pub unsafe fn map_kernel_sections(
    page_table: &mut Sv39PageTable<FrameAllocator>,
) -> Result<(), PagingError> {
    unsafe {
        map_section(
            page_table,
            &__stext,
            &__etext,
            MappingFlags::READ | MappingFlags::EXECUTE,
        )?;

        map_section(page_table, &__srodata, &__erodata, MappingFlags::READ)?;

        map_section_size(
            page_table,
            &__sdata,
            0x400_000, // 4MB
            MappingFlags::READ | MappingFlags::WRITE,
        )?;
    }

    Ok(())
}

pub struct FrameAllocator {
    start: PhysAddr,
    end: PhysAddr,
    next: usize,
}

// Size of the RAM in bytes (2MB)
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
    let ram_start; 
    unsafe {
        ram_start = addr(&_stack_start);
    };
    unsafe { FrameAllocator::init(ram_start, RAM_SIZE) };
    println!("Finished initializing frame allocator");
}

pub unsafe fn init_page_table() -> Sv39PageTable<FrameAllocator> {
    println!("Initializing page table");
    let mut page_table = Sv39PageTable::<FrameAllocator>::try_new().unwrap();

    unsafe {
        if let Err(e) = map_kernel_sections(&mut page_table) {
            panic!("Failed to map kernel sections: {:?}", e);
        }
    }

    // Map serial port for uart
    let _ = page_table
        .map(
            VirtAddr::from_usize(0x1000_0000),
            PhysAddr::from_usize(0x1000_0000),
            PageSize::Size4K,
            MappingFlags::READ | MappingFlags::WRITE,
        )
        .unwrap();

    let mut reg = Satp::from_bits(0);
    reg.set_mode(Mode::Sv39);
    let ppn = page_table.root_paddr().as_usize() >> 12;
    reg.set_ppn(ppn);
    println!("Turn on MMU");
    println!(
        "Page table root PhysAddr: {:#x}, aligned: {}",
        page_table.root_paddr().as_usize(),
        page_table.root_paddr().as_usize() % PAGE_SIZE_4K
    );
    println!("Satp bits: {:#x}", reg.bits());
    println!("Satp PPN: {:#x}", reg.ppn());
    println!("stvec: {:#x}", riscv::register::stvec::read().bits());
    riscv::asm::sfence_vma_all();
    unsafe { satp::write(reg) };
    println!("Finished initializing page table");
    page_table
}

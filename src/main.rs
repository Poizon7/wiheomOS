#![no_main]
#![no_std]

use core::panic::PanicInfo;
use alloc::boxed::Box;
use riscv::register::satp::Satp;
extern crate alloc;

pub mod serial;
mod exception;
mod interrupt;
mod page;
mod allocator;
mod device_tree;

#[riscv_rt::entry]
fn main() -> ! {
    // Load the device tree pointer from a1 register
    // This needs to be the first instruction 
    // to ensure the device tree ptr is in register
    let dtb = device_tree::dtb_ptr();

    println!("Hello World!");
    device_tree::init(dtb);

    unsafe { page::init_frame_allocator() };
    let mut page_table = unsafe { page::init_page_table() };
    allocator::init_heap(&mut page_table).unwrap();
    interrupt::interrupt_init();

    println!("Initializing done");

    let mut reg = Satp::from_bits(0);
    println!("{:?}", reg.mode());

    loop {
        riscv::asm::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

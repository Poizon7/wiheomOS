#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod exception;
mod interrupt;
pub mod serial;

#[riscv_rt::entry]
fn main() -> ! {
    println!("Hello World!");

    interrupt::interrupt_init();

    println!("Initializing done");
    loop {
        riscv::asm::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

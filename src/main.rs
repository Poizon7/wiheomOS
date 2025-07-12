#![no_main]
#![no_std]

use core::panic::PanicInfo;
use core::str;

use crate::serial::SERIAL1;

pub mod serial;
mod exception;

// Use `main` as the entry point of this application, which may not return.
#[riscv_rt::entry]
fn main() -> ! {
    // Now the serial port is ready to be used. To send a byte:
    println!("Hello World!");

    loop {
        let mut data = [0; 16];
        let mut i = 0;

        // To receive a byte:
        loop {
            let byte = SERIAL1.lock().receive();
            if byte == 13 {
                // ASCII code for carriage return
                break;
            }
            print!("{}", byte as char);
            data[i] = byte;
            i += 1;
        }

        println!();
        let text = unsafe { str::from_utf8_unchecked(&data) };

        println!("{}", text);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

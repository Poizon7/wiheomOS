#![no_main]
#![no_std]

use core::fmt::Write;

// make sure the panic handler is linked in
extern crate panic_halt;
use uart_16550::MmioSerialPort;

const SERIAL_PORT_BASE_ADDRESS: usize = 0x1000_0000;

// Use `main` as the entry point of this application, which may not return.
#[riscv_rt::entry]
fn main() -> ! {
    let mut serial_port = unsafe { MmioSerialPort::new(SERIAL_PORT_BASE_ADDRESS) };
    serial_port.init();

    // Now the serial port is ready to be used. To send a byte:
    writeln!(serial_port, "Hello World!").ok();

    loop {
        let mut data = [0; 16];
        let mut i = 0;

        // To receive a byte:
        loop {
            let byte = serial_port.receive();
            if byte == 13 {
                // ASCII code for carriage return
                break;
            }
            serial_port.send(byte);
            data[i] = byte;
            i += 1;
        }
        serial_port.send(13);
        serial_port.send(10);

        for i in data {
            serial_port.send(i);
        }
        serial_port.send(13);
        serial_port.send(10);
    }
}

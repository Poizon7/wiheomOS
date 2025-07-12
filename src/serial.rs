use uart_16550::MmioSerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

const SERIAL_PORT_BASE_ADDRESS: usize = 0x1000_0000;

lazy_static! {
    pub static ref SERIAL1: Mutex<MmioSerialPort> = {
        let mut serial_port = unsafe { MmioSerialPort::new(SERIAL_PORT_BASE_ADDRESS) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n")};
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}


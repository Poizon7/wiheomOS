use core::arch::asm;
use riscv::{
    interrupt::Interrupt,
    register::{
        sie::{self, Sie},
        sstatus, time,
    },
};

use crate::println;

pub fn interrupt_init() {
    unsafe {
        sstatus::set_sie();
    }
    let mut _sie: Sie = Sie::from_bits(0);
    _sie.set_stimer(true);
    _sie.set_ssoft(true);
    _sie.set_sext(true);

    unsafe {
        sie::write(_sie);
    }

    unsafe {
        asm!(
            "rdtime t0",
            "li t1, 10000000",
            "add t0, t0, t1",
            "li a7, 0x54494d45",
            "li a6, 0x00000000",
            "ecall",
        )
    }
}

#[riscv_rt::core_interrupt(Interrupt::SupervisorTimer)]
fn supervisor_timer_handler() {
    println!("Supervisor Timer Interrupt");

    // Read the current time from stimer
    let now = time::read64();
    println!("Current time: {}", now);
    unsafe {
        asm!(
            "rdtime t0",
            "li t1, 10000000",
            "add a0, t0, t1",
            "li a7, 0x54494d45",
            "li a6, 0x00000000",
            "ecall",
        )
    }
}

#[unsafe(export_name = "DefaultHandler")]
unsafe fn interrupt_handler(interrupt: Interrupt) {
    println!("Interrupt: {:?}", interrupt);
}

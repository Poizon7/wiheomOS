use riscv::interrupt::Exception;

use crate::println;

#[riscv_rt::exception(Exception::InstructionMisaligned)]
fn instruction_misaligned_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Instruction Misaligned: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::InstructionFault)]
fn instruction_fault_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Instruction Fault: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::IllegalInstruction)]
fn illigal_instruction_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Illigal Instruction: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::Breakpoint)]
fn breakpoint_handler(trap_frame: &riscv_rt::TrapFrame) {
    println!("Breakpoint: {:?}", trap_frame);
}

#[riscv_rt::exception(Exception::LoadMisaligned)]
fn load_misaligned_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Load Misaligned: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::LoadFault)]
fn load_fault_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Load Fault: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::StoreMisaligned)]
fn store_misaligned_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Store Misaligned: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::StoreFault)]
fn store_fault_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Store Fault: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::UserEnvCall)]
fn user_env_call_handler(trap_frame: &riscv_rt::TrapFrame) {
    println!("User Env Call: {:?}", trap_frame);
}

#[riscv_rt::exception(Exception::SupervisorEnvCall)]
fn supervisor_env_call_handler(trap_frame: &riscv_rt::TrapFrame) {
    println!("Supervisor Env Call: {:?}", trap_frame);
}

#[riscv_rt::exception(Exception::InstructionPageFault)]
fn instruction_page_fault_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Instruction Page Fault: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::LoadPageFault)]
fn load_page_fault_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Load Page Fault: {:?}", trap_frame);
    loop {}
}

#[riscv_rt::exception(Exception::StorePageFault)]
fn store_page_fault_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("Store Page Fault: {:?}", trap_frame);
    loop {}
}

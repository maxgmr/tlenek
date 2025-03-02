#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use lazy_static::lazy_static;
use tlenek_core::{
    gdt::{self, DOUBLE_FAULT_IST_INDEX},
    hlt_loop,
    qemu::{exit_qemu, QemuExitCode},
    serial_print, serial_println,
};
use volatile::Volatile;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow :(");
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

// Trigger a stack overflow
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    Volatile::new(0).read(); // prevent tail recursion optimisation
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    tlenek_core::test_panic_handler(info)
}

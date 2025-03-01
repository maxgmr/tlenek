//! Core library for the [tlenek](https://github.com/maxgmr/tlenek) x86_64 operating system.
#![warn(missing_docs)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test_framework::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod gdt;
pub mod interrupts;
pub mod qemu;
pub mod serial;
pub mod test_framework;
pub mod vga_text;

pub use test_framework::{test_panic_handler, test_runner};

/// General initialisation routines.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn breakpoint_exception() {
        // invoke a breakpoint exception
        x86_64::instructions::interrupts::int3();
    }
}

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
#[cfg(not(test))]
use tlenek_core::vga_text::{set_vga_attr, VgaBgColour};
use tlenek_core::{
    println,
    vga_text::{set_vga_fg, VgaFgColour},
};
#[cfg(test)]
use tlenek_core::{test_panic_handler, test_runner};

const VERSION_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

/// Entry point.
///
/// Use Linux conventions- make sure it's called `_start`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    set_vga_fg(VgaFgColour::LightGreen);
    println!("{}", VERSION_MSG);

    #[cfg(test)]
    test_main();

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // format & print panic message
    set_vga_attr(VgaBgColour::default(), VgaFgColour::LightRed, false);
    println!("{}", info);
    // ...loop forever...
    loop {}
}

/// Called on test panic.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

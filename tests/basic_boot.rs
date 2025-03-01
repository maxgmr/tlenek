#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(tlenek_core::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use tlenek_core::{println, test_panic_handler};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    // The compiler doesn't know QEMU exits after testing
    #[allow(clippy::empty_loop)]
    loop {}
}

// This test is important because it's called in a basic environment without any initialisation
// routines. It's crucial that println works out-of-the-box!
#[test_case]
fn test_println() {
    println!("testing, testing, 1-2-3!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

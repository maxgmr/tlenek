#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// Entry point.
///
/// Use Linux conventions- make sure it's called `_start`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

/// Called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // ...loop forever...
    loop {}
}

#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};
use tlenek_core::vga_text::{VgaBgColour, VgaFgColour, WRITER};

const VERSION_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

/// Entry point.
///
/// Use Linux conventions- make sure it's called `_start`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    WRITER.lock().set_fg(VgaFgColour::LightGreen);
    WRITER.lock().write_str(VERSION_MSG).unwrap();
    WRITER.lock().write_str("\n").unwrap();
    WRITER
        .lock()
        .set_attr(VgaBgColour::Blue, VgaFgColour::Red, true);
    WRITER.lock().write_str(":)").unwrap();
    WRITER
        .lock()
        .set_attr(VgaBgColour::default(), VgaFgColour::Pink, false);
    WRITER.lock().write_str("\n\n").unwrap();
    write!(WRITER.lock(), "1/2 = {}", 1.0 / 2.0).unwrap();

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // ...loop forever...
    loop {}
}

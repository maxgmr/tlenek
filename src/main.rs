#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_text;

use vga_text::{print_str, VgaBgColour, VgaFgColour};

const MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

const BG_COLOUR: VgaBgColour = VgaBgColour::Black;
const FG_COLOUR: VgaFgColour = VgaFgColour::LightGreen;
const BLINK: bool = false;

/// Entry point.
///
/// Use Linux conventions- make sure it's called `_start`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print_str(MSG, BG_COLOUR, FG_COLOUR, BLINK);

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // ...loop forever...
    loop {}
}

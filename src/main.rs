#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_text;

use vga_text::{print_str, VgaBgColour, VgaFgColour};

static MSG: &str = "Tlenek v0.1.0-alpha.1";
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

#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_text;

use vga_text::{putc_vga_text, VgaBgColour, VgaFgColour, VGA_BUF_ADDR};

static MSG: &[u8] = b"Tlenek v0.1.0-alpha.1";
const BG_COLOUR: VgaBgColour = VgaBgColour::Black;
const FG_COLOUR: VgaFgColour = VgaFgColour::LightGreen;
const BLINK: bool = false;

/// Entry point.
///
/// Use Linux conventions- make sure it's called `_start`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buf = VGA_BUF_ADDR as *mut u8;

    for (i, &byte) in MSG.iter().enumerate() {
        unsafe {
            putc_vga_text(vga_buf, i, byte, BG_COLOUR, FG_COLOUR, BLINK);
        }
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // ...loop forever...
    loop {}
}

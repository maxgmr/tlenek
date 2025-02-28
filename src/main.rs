#![no_std]
#![no_main]

use core::panic::PanicInfo;
use tlenek_core::{
    print, println,
    vga_text::{set_vga_attr, set_vga_bg, set_vga_blink, set_vga_fg, VgaBgColour, VgaFgColour},
};

const VERSION_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

/// Entry point.
///
/// Use Linux conventions- make sure it's called `_start`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    set_vga_fg(VgaFgColour::LightGreen);
    println!("{}", VERSION_MSG);
    set_vga_attr(VgaBgColour::Blue, VgaFgColour::Red, true);
    print!(":)");
    set_vga_bg(VgaBgColour::Black);
    set_vga_blink(false);
    println!();
    println!();
    set_vga_fg(VgaFgColour::Pink);
    println!("1/2 = {}", 1.0 / 2.0);

    panic!("I'm panicking because of how great things are!");

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // format & print panic message
    set_vga_attr(VgaBgColour::default(), VgaFgColour::LightRed, false);
    println!("{}", info);
    // ...loop forever...
    loop {}
}

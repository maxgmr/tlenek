#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
#[cfg(not(test))]
use tlenek_core::vga_text::VgaBgColour;
use tlenek_core::{
    hlt_loop, init, print, println,
    vga_text::{
        set_default_vga_attr, set_vga_attr, set_vga_fg, vga_bg, vga_blink, vga_fg, VgaFgColour,
    },
};
#[cfg(test)]
use tlenek_core::{test_panic_handler, test_runner};

const VERSION_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

// Allow main kernel entry point to be type-checked to avoid UB
entry_point!(kernel_main);

/// Entry point.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::structures::paging::Translate;

    init();
    welcome();

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { tlenek_core::memory::init(phys_mem_offset) };

    let addresses = [
        tlenek_core::vga_text::VGA_BUFFER_ADDR as u64,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1A10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = x86_64::VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    hlt_loop();
}

/// Called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // format & print panic message
    set_vga_attr(VgaBgColour::default(), VgaFgColour::LightRed, false);
    println!("{}", info);
    // ...loop forever...
    hlt_loop();
}

/// Called on test panic.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// Friendly welcome message.
fn welcome() {
    let old_bg = vga_bg();
    let old_fg = vga_fg();
    let old_blink = vga_blink();

    set_default_vga_attr();
    print!("Welcome to ");
    set_vga_fg(VgaFgColour::LightGreen);
    print!("{}", VERSION_MSG);
    set_default_vga_attr();
    println!("!");

    // Clean up after yourself!
    set_vga_attr(old_bg, old_fg, old_blink);
}

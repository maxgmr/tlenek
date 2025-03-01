//! Functionality related to interrupts.

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{
    print, println,
    vga_text::{set_vga_fg, vga_fg, VgaFgColour},
};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

/// Create a new [InterruptDescriptorTable], which specifies handler functions for each CPU
/// exception.
pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // Output a message & pretty-print the stack frame.
    let old_fg = vga_fg();
    set_vga_fg(VgaFgColour::LightRed);
    print!("EXCEPTION");
    set_vga_fg(old_fg);
    println!(": BREAKPOINT\n{:#?}", stack_frame);
}

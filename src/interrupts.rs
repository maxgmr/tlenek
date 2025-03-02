//! Functionality related to interrupts.

use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

use crate::{
    gdt::DOUBLE_FAULT_IST_INDEX,
    print, println,
    vga_text::{set_vga_fg, vga_fg, VgaFgColour},
};

const PS2_CONTROLLER_PORT: u16 = 0x60;

const PIC_INTERRUPT_LINES: u8 = 8;

/// Start after the 32 exception slots
pub const PIC_1_OFFSET: u8 = 32;
/// Start after PIC 1
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + PIC_INTERRUPT_LINES;

/// The different PIC hardware interrupts.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// Programmable interval timer (PIT) interrupt
    Timer = PIC_1_OFFSET,
    /// Keyboard interrupt
    Keyboard,
}
impl From<InterruptIndex> for u8 {
    fn from(value: InterruptIndex) -> Self {
        value as u8
    }
}
impl From<InterruptIndex> for usize {
    fn from(value: InterruptIndex) -> Self {
        u8::from(value) as usize
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        use InterruptIndex::*;

        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // UNSAFE: This is safe because `DOUBLE_FAULT_IST_INDEX` is valid and not already used for
        // another exception.
        unsafe {
        idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt.machine_check.set_handler_fn(machine_check_handler);

        // PIC hardware interrupts
        idt[Timer.into()].set_handler_fn(timer_interrupt_handler);
        idt[Keyboard.into()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

/// The Intel 8259 primary/secondary PIC layout used for hardware interrupts.
pub static PICS: spin::Mutex<ChainedPics> =
    // UNSAFE: Can cause UB if the PIC is misconfigured.
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Create a new [InterruptDescriptorTable], which specifies handler functions for each CPU
/// exception.
pub fn init_idt() {
    IDT.load();
}

/// Handler for breakpoints.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    exception_title();
    println!("BREAKPOINT\n{:#?}", stack_frame);
}

/// Handler for double faults. Invoked when the CPU fails to invoke an exception handler. Used to
/// avoid the horrifying triple fault, which causes a full system reset!
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    // Error code is always 0 for double faults
    _error_code: u64,
) -> ! {
    exception_title();
    // Must diverge- x86_64 prevents returning from a double fault.
    panic!("DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Handler for machine check. Unrecoverable- invoked when the processor detects internal errors
/// (bad memory, bus errors, cache errors, etc.).
extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    exception_title();
    panic!("MACHINE_CHECK\n{:#?}", stack_frame)
}

/// Handler for the hardware timer interrupt.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        send_eoi(InterruptIndex::Timer);
    }
}

/// Handler for the hardware keyboard interrupt.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            spin::Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    // Read scancode
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(PS2_CONTROLLER_PORT);
    let scancode: u8 = unsafe { port.read() };

    // Print key
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                // DecodedKey::Unicode(character) => print!("{:#?}", character),
                DecodedKey::Unicode(character) => print!("{}", character),
                // DecodedKey::RawKey(key) => print!("{:?}", key),
                DecodedKey::RawKey(_) => (),
            }
        }
    }

    unsafe {
        send_eoi(InterruptIndex::Keyboard);
    }
}

/// Send end of interrupt signal
/// UNSAFE: Using the wrong interrupt vector number could delete an important unsent interrupt
/// or cause the system to hang.
unsafe fn send_eoi(interrupt_index: InterruptIndex) {
    PICS.lock().notify_end_of_interrupt(interrupt_index.into());
}

/// Print "EXCEPTION" in nice scary red text.
fn exception_title() {
    let old_fg = vga_fg();
    set_vga_fg(VgaFgColour::LightRed);
    print!("EXCEPTION");
    set_vga_fg(old_fg);
    print!(": ");
}

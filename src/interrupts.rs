//! Functionality related to interrupts.

use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin;
use x86_64::{
    instructions::port::Port,
    registers,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use crate::{
    gdt::DOUBLE_FAULT_IST_INDEX,
    hlt_loop, print, println,
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

        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        // UNSAFE: This is safe because `DOUBLE_FAULT_IST_INDEX` is valid and not already used for
        // another exception.
        unsafe {
        idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.cp_protection_exception.set_handler_fn(cp_protection_exception_handler);
        idt.hv_injection_exception.set_handler_fn(hv_injection_exception_handler);
        idt.vmm_communication_exception.set_handler_fn(vmm_communication_exception_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);

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

/// Handler for division error.
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "DIVIDE ERROR");
}

/// Handler for debug.
/// Logic from [OSDev Wiki](https://wiki.osdev.org/Exceptions#Debug)
extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    let exception_name = "DEBUG";

    use registers::debug::Dr6Flags;
    use registers::debug::Dr7Flags;

    let dr6 = registers::debug::Dr6::read();
    let dr7 = registers::debug::Dr7::read().flags();

    // This is super ugly...
    if dr6.contains(Dr6Flags::ACCESS_DETECTED) {
        // Fault: Debug register access violation.
        generic_fault_handler(stack_frame, exception_name);
    } else if dr6.contains(Dr6Flags::TRAP0) {
        if !dr7.contains(Dr7Flags::LOCAL_BREAKPOINT_0_ENABLE)
            && !dr7.contains(Dr7Flags::GLOBAL_BREAKPOINT_0_ENABLE)
        {
            // Fault: Execute
            generic_fault_handler(stack_frame, exception_name);
        } else {
            // Trap: write or read/write
            generic_trap_handler(stack_frame, exception_name);
        }
    } else if dr6.contains(Dr6Flags::TRAP1) {
        if !dr7.contains(Dr7Flags::LOCAL_BREAKPOINT_1_ENABLE)
            && !dr7.contains(Dr7Flags::GLOBAL_BREAKPOINT_1_ENABLE)
        {
            // Fault: Execute
            generic_fault_handler(stack_frame, exception_name);
        } else {
            // Trap: write or read/write
            generic_trap_handler(stack_frame, exception_name);
        }
    } else if dr6.contains(Dr6Flags::TRAP2) {
        if !dr7.contains(Dr7Flags::LOCAL_BREAKPOINT_2_ENABLE)
            && !dr7.contains(Dr7Flags::GLOBAL_BREAKPOINT_2_ENABLE)
        {
            // Fault: Execute
            generic_fault_handler(stack_frame, exception_name);
        } else {
            // Trap: write or read/write
            generic_trap_handler(stack_frame, exception_name);
        }
    } else if dr6.contains(Dr6Flags::TRAP3) {
        if !dr7.contains(Dr7Flags::LOCAL_BREAKPOINT_3_ENABLE)
            && !dr7.contains(Dr7Flags::GLOBAL_BREAKPOINT_2_ENABLE)
        {
            // Fault: Execute
            generic_fault_handler(stack_frame, exception_name);
        } else {
            // Trap: write or read/write
            generic_trap_handler(stack_frame, exception_name);
        }
    } else if dr6.contains(Dr6Flags::STEP) {
        // Trap: Single-step
        generic_trap_handler(stack_frame, exception_name);
    } else if dr6.contains(Dr6Flags::SWITCH) {
        // Trap: Task-switch
        generic_trap_handler(stack_frame, exception_name);
    }

    // Clear DR6
    unsafe {
        core::arch::asm!("mov dr6, {}", in(reg) 0_u64);
    }
}

/// Handler for non-maskable interrupts.
extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_trap_handler(stack_frame, "NON-MASKABLE INTERRUPT");
}

/// Handler for breakpoints.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    generic_trap_handler(stack_frame, "BREAKPOINT");
}

/// Handler for overflows.
extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_trap_handler(stack_frame, "OVERFLOW");
}

/// Handler for exceeding bound range.
extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "BOUND RANGE EXCEEDED");
}

/// Handler for invalid opcode faults.
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "INVALID OPCODE");
}

/// Handler for when an FPU instruction is attempted but there is no FPU.
extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "DEVICE NOT AVAILABLE");
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

/// Handler for invalid TSS.
extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("INVALID TSS {:#X}\n{:#?}", error_code, stack_frame);
    }
}

/// Handler for segment not present exceptions.
extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("SEGMENT NOT PRESENT {:#X}\n{:#?}", error_code, stack_frame);
    }
}

/// Handler for stack-segment faults.
extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("STACK-SEGMENT FAULT {:#X}\n{:#?}", error_code, stack_frame);
    }
}

/// Handler for general protection faults.
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!(
            "GENERAL PROTECTION FAULT {:#X}\n{:#?}",
            error_code, stack_frame
        );
    }
}

/// Handler for page faults.
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!(
            "PAGE FAULT\nError Code: {:?}\nAccessed Address: {:#X} \n{:#?}",
            error_code,
            registers::control::Cr2::read(),
            stack_frame
        );
    }
    // Can't continue execution without resolving the page fault
    hlt_loop();
}

/// Handler for x87 floating-point exceptions.
extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "x87 FLOATING-POINT");
}

/// Handler for alignment check exceptions.
extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("ALIGNMENT CHECK {:#X}\n{:#?}", error_code, stack_frame);
    }
}

/// Handler for machine check. Unrecoverable- invoked when the processor detects internal errors
/// (bad memory, bus errors, cache errors, etc.).
extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    exception_title();
    panic!("MACHINE_CHECK\n{:#?}", stack_frame)
}

/// Handler for SIMD floating-point exceptions.
extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "SIMD FLOATING-POINT");
}

/// Handler for virtualization exceptions.
extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "VIRTUALIZATION");
}

/// Handler for control protection exceptions.
extern "x86-interrupt" fn cp_protection_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("CONTROL PROTECTION {:#X}\n{:#?}", error_code, stack_frame);
    }
}

/// Handler for hypervisor injection exceptions
extern "x86-interrupt" fn hv_injection_exception_handler(stack_frame: InterruptStackFrame) {
    // TODO
    generic_fault_handler(stack_frame, "HYPERVISOR INJECTION");
}

/// Handler for VMM communication exceptions.
extern "x86-interrupt" fn vmm_communication_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("VMM COMMUNICATION {:#X}\n{:#?}", error_code, stack_frame);
    }
}

/// Handler for security exceptions.
extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("SECURITY {:#X}\n{:#?}", error_code, stack_frame);
    }
}

/// Generic behaviour for fault errors (print exception on debug only, else do nothing)
fn generic_fault_handler(_stack_frame: InterruptStackFrame, exception_name: &'static str) {
    #[cfg(debug_assertions)]
    {
        exception_title();
        println!("{}\n{:#?}", exception_name, _stack_frame);
    }
}

/// Generic behaviour for trap errors (print exception and continue)
fn generic_trap_handler(stack_frame: InterruptStackFrame, exception_name: &'static str) {
    exception_title();
    println!("{}\n{:#?}", exception_name, stack_frame);
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
    print!("\nEXCEPTION");
    set_vga_fg(old_fg);
    print!(": ");
}

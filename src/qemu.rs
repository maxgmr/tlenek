//! QEMU-related functionality.

const ISA_DEBUG_EXIT_IOBASE: u16 = 0xF4;

/// The different QEMU exit codes
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
#[allow(missing_docs)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

/// Exit QEMU.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    // UNSAFE: Writing to an I/O port can generally result in UB.
    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_IOBASE);
        port.write(exit_code as u32);
    }
}

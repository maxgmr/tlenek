//! Test functionality.

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::BootInfo;

use crate::{
    hlt_loop,
    qemu::{exit_qemu, QemuExitCode},
    serial_print, serial_println,
};

/// [Testable] types can be run as tests (i.e. they should panic if their test fails).
pub trait Testable {
    /// Should panic if the test fails.
    fn run(&self);
}
impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// The test runner.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} test(s)...", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Displays failure and panic msg.
pub fn test_panic_handler(info: &PanicInfo<'_>) -> ! {
    serial_println!("[FAIL]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);

    hlt_loop();
}

#[allow(missing_docs)]
mod test_entry_point {
    #[cfg(test)]
    bootloader::entry_point!(super::test_kernel_main);
}

/// Test entry point.
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    super::init();
    super::test_main();
    hlt_loop();
}

/// Test panic handler.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

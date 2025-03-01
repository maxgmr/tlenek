//! Functionality related to the [global descriptor table](https://en.wikipedia.org/wiki/Global_Descriptor_Table).
//!
//! See [GDT] for more info.

use lazy_static::lazy_static;
use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

/// Index of the interrupt stack table. Used to get a good stack in the case of a double fault.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    /// The global descriptor table.
    ///
    /// The GDT was used to isolate programs from each other before paging.
    ///
    /// Segmentation is no longer supported in 64-bit x86_64, but the GDT still exists.
    ///
    /// The GDT is now mostly used for switching between kernel/user space and loading a TSS structure.
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    /// The task state segment.
    ///
    /// In x86_64, holds two stack tables:
    ///
    /// 1. The privilege stack table: Used by the CPU when the privilege level changes.
    /// 2. The interrupt stack table: A table of 7 pointers to known-good stacks. Allows the CPU to
    ///    switch to a good stack when an exception occurs, because the CPU needs to push the
    ///    exception stack frame _somewhere_ even if a stack overflow causes a page fault.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            // TODO no memory management yet. Use a `static mut` array as stack storage for now.
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE // return end of stack
        };
        tss
    };
}

/// Initialise the [GDT].
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    // UNSAFE: Possible to break memory safety by loading invalid selectors. Here, we've loaded
    // valid selectors.
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

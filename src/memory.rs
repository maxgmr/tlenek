//! Memory-related functionality.

use x86_64::{
    registers::control::Cr3,
    structures::paging::{page_table::FrameError, OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
};

/// Initialise a new [OffsetPageTable].
///
/// # Safety
///
/// - Caller must guarantee that the complete physical memory is mapped to virtual memory at
///   the passed `physical_memory_offset`.
/// - This function must be only called once to avoid aliasing `&mut` references (which is UB).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Get a mutable reference to the level 4 page table.
///
/// # Safety
///
/// Calling this function multiple times can cause UB, so it should only be called from the [init]
/// function.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let physical_start_addr = level_4_table_frame.start_address();
    let virt = physical_memory_offset + physical_start_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

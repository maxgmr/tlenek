//! Memory-related functionality.

use x86_64::{registers::control::Cr3, structures::paging::PageTable, VirtAddr};

/// Return a mutable reference to the active level 4 table.
///
/// # Safety
///
/// - Caller must guarantee that the complete physical memory is mapped to virtual memory at
///   the passed `physical_memory_offset`.
/// - This function must by only called once to avoid aliasing `&mut` references (which is UB).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let physical_start_addr = level_4_table_frame.start_address();
    let virt = physical_memory_offset + physical_start_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

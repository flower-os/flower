use super::*;
use core::alloc::{Layout, GlobalAlloc};

pub fn map_new_process() -> InactivePageMap {
    let mut temporary_page = TemporaryPage::new();

    // This must be duplicated to avoid double locks. This is safe though -- in this context!
    let mut active_table = unsafe { ActivePageMap::new() };
    let frame = PhysicalAddress(PHYSICAL_ALLOCATOR.allocate(0).expect("no more frames") as usize);
    let mut new_table = InactivePageMap::new(frame, &mut active_table, &mut temporary_page);

    let kernel_pml4_entry = active_table.p4()[511];

    let mut table = unsafe {
        temporary_page.map_table_frame(frame.clone(), &mut active_table)
    };

    // Copy kernel pml4 entry
    table[511] = kernel_pml4_entry;

    unsafe {
        temporary_page.unmap(&mut active_table);
    }

    // Drop this lock so that the RAII guarded temporary page can be destroyed
    drop(active_table);

    new_table
}

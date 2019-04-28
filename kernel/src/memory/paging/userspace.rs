use super::*;
use crate::userspace::{STACK_TOP, INITIAL_STACK_SIZE_PAGES};
use core::alloc::{Layout, GlobalAlloc};

pub fn map_new_process() -> InactivePageMap {
    let mut temporary_page = TemporaryPage::new();

    // This must be duplicated to avoid double locks. This is safe though -- in this context!
    let mut active_table = unsafe { ActivePageMap::new() };
    let frame = PhysicalAddress(PHYSICAL_ALLOCATOR.allocate(0).expect("no more frames") as usize);
    let mut new_table = InactivePageMap::new(frame, &mut active_table, &mut temporary_page);

    // Copy kernel pml4 entry
    let kernel_pml4_entry = active_table.p4()[511];
    let mut table = unsafe {
        temporary_page.map_table_frame(frame.clone(), &mut active_table)
    };

    table[511] = kernel_pml4_entry;

    unsafe {
        temporary_page.unmap(&mut active_table);
    }

    // Set up user stack
    let stack_top = Page::containing_address(STACK_TOP, PageSize::Kib4);
    let stack_bottom = stack_top - INITIAL_STACK_SIZE_PAGES;
    trace!("stack bottom = 0x{:x}", stack_bottom.start_address().unwrap());

    active_table.with_inactive_p4(&mut new_table, &mut temporary_page, |mapper| {
        unsafe {
            mapper.map_range(
                stack_bottom..=stack_top,
                EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE | EntryFlags::NO_EXECUTE,
                InvalidateTlb::NoInvalidate,
                ZeroPage::NoZero,
            );
        }
    });

    // Drop this lock so that the RAII guarded temporary page can be destroyed
    drop(active_table);

    new_table
}

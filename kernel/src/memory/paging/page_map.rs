// Taken from https://os.phil-opp.com/remap-the-kernel/
// Many thanks!

use super::*;
use core::ops::{Deref, DerefMut};
use core::ops::RangeInclusive;

pub struct Mapper {
    p4: Unique<PageTable<Level4>>,
}

impl Mapper {
    const unsafe fn new() -> Self {
        Mapper {
            // The address points to the recursively mapped entry in the P4 table, which we can use
            // to access the P4 table itself.
            p4: Unique::new_unchecked(0xffffffff_fffff000 as *mut _),
        }
    }

    fn p4(&self) -> &PageTable<Level4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe { self.p4.as_mut() }
    }

    /// Walks the page tables and translates this page into a physical address
    pub fn walk_page_table(&self, page: Page) -> Option<(PhysicalAddress, PageSize)> {
        let p3 = self.p4().next_page_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                // 1GiB page
                let p3_entry = &p3[page.p3_index()];
                if p3_entry.physical_address().is_some() {
                    if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                        panic!("1 GiB pages are not supported!");
                    }
                }

                if let Some(p2) = p3.next_page_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    // 2MiB page
                    if let Some(start_frame) = p2_entry.physical_address() {
                        if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            // Check that the address is 2MiB aligned
                            assert_eq!(
                                start_frame.0 >> 12 % PAGE_TABLE_ENTRIES,
                                0,
                                "Adress is not 2MiB aligned!"
                            );
                            return Some((
                                PhysicalAddress(start_frame.0 >> 12 + page.p1_index()),
                                PageSize::Mib2,
                            ));
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_page_table(page.p3_index()))
            .and_then(|p2| p2.next_page_table(page.p2_index()))
            .and_then(|p1| Some((p1[page.p1_index()].physical_address()?, PageSize::Kib4)))
            .or_else(huge_page)
    }

    pub unsafe fn map_to(
        &mut self,
        page: Page,
        physical_address: PhysicalAddress,
        flags: EntryFlags,
        invplg: bool
    ) {
        use self::EntryFlags;

        let mut p2 = self.p4_mut()
            .next_table_create(page.p4_index()).expect("No next p3 table!")
            .next_table_create(page.p3_index()).expect("No next p2 table!");

        assert!(page.size.is_some(), "Page to map requires size!");

        if page.size.unwrap() == PageSize::Kib4 {
            let mut p1 = p2.next_table_create(page.p2_index())
                .expect("No next p1 table!");

            // 4kib page
            p1[page.p1_index()].set(
                physical_address,
                flags | EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );

            if invplg {
                tlb::flush(::x86_64::VirtualAddress(page.start_address().unwrap()));
            }
        } else {
            panic!("2mib pages are only partially supported!");
        }
    }

    pub unsafe fn map(&mut self, page: Page, flags: EntryFlags, invplg: bool) {
        use core::ptr;

        assert!(page.size.is_some(), "Page needs size!");
        let order = if page.size.unwrap() == PageSize::Kib4 {
            0
        } else {
            9
        };

        let ptr = PHYSICAL_ALLOCATOR.allocate(order).expect("Out of physical memory!");
        let frame = PhysicalAddress(ptr as usize);
        self.map_to(page, frame, flags, invplg);

        // Zero the page
        ptr::write_bytes(
            page.start_address().unwrap() as *mut u8,
            0,
            page.size.unwrap().bytes()
        );
    }

    pub unsafe fn unmap(&mut self, page: Page, free_physical_memory: bool, invplg: bool) {
        assert!(page.start_address().is_some(), "Page to map requires size!");
        assert!(
            self.walk_page_table(page).is_some(),
            "Virtual address 0x{:x} is not mapped!",
            page.start_address().unwrap()
        );

        let mut p2 = self.p4_mut()
            .next_page_table_mut(page.p4_index()).expect("Unmap called on unmapped page!")
            .next_page_table_mut(page.p3_index()).expect("Unmap called on unmapped page!");

        let p1 = p2.next_page_table_mut(page.p2_index());

        if let Some(p1) = p1 {
            // 4kib page

            let frame = p1[page.p1_index()].physical_address().expect("Page already unmapped!");
            p1[page.p1_index()].set_unused();

            // TODO free p1/p2/p3 tables if they are empty
            if free_physical_memory {
                PHYSICAL_ALLOCATOR.deallocate(frame.0 as *const _, 0);
            }
        } else {
            // Huge 2mib page

            let frame = p2[page.p2_index()].physical_address().expect("Page already unmapped!");
            p2[page.p2_index()].set_unused();

            // TODO free p2/p3 tables if they are empty
            if free_physical_memory {
                PHYSICAL_ALLOCATOR.deallocate(frame.0 as *const _, 9);
            }
        }

        if invplg {
            // Flush tlb
            tlb::flush(::x86_64::VirtualAddress(page.start_address().unwrap()));
        }
    }

    /// Identity maps a range of addresses as 4 kib pages
    pub unsafe fn id_map_range(
        &mut self,
        addresses: RangeInclusive<usize>,
        flags: EntryFlags,
        invplg: bool
    ) {
        for frame_no in (addresses.start() / 4096)..=(addresses.end() / 4096) {
            let addr = (frame_no * 4096) as usize;
            self.map_to(
                Page::containing_address(addr, PageSize::Kib4),
                PhysicalAddress(addr as usize),
                flags,
                invplg,
            );
        }
    }


}

pub struct TemporaryPage {
    page: Page,
}

impl TemporaryPage {
    pub fn new(page: Page) -> TemporaryPage {
        TemporaryPage { page }
    }

    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub unsafe fn map(
        &mut self,
        frame: PhysicalAddress,
        active_table:
        &mut ActivePageMap
    ) -> VirtualAddress {
        let page_addr = self.page.start_address().expect("Temporary page requires size");
        assert!(
            active_table.walk_page_table(self.page).is_none(),
            "Temporary page at 0x{:x} is already mapped",
            page_addr
        );

        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, true);
        VirtualAddress(self.page.start_address().expect("Page in TemporaryPage requires size"))
    }

    /// Unmaps the temporary page in the active table.
    pub unsafe fn unmap(&mut self, active_table: &mut ActivePageMap) {
        // Unmap and invplg but do not free backing mem
        active_table.unmap(self.page, false, true);
    }

    pub unsafe fn map_table_frame(
        &mut self,
        frame: PhysicalAddress,
        active_table: &mut ActivePageMap
    ) -> &mut PageTable<Level1> {
        &mut *(self.map(frame, active_table).0 as *mut PageTable<Level1>)
    }
}

pub struct ActivePageMap {
    mapper: Mapper,
}

impl ActivePageMap {
    pub const unsafe fn new() -> ActivePageMap {
        ActivePageMap {
            mapper: Mapper::new()
        }
    }

    pub fn with_inactive_p4<F: FnOnce(&mut ActivePageMap) -> R, R>(
        &mut self,
        table: &mut InactivePageMap,
        temporary_page: &mut TemporaryPage,
        f: F
    ) -> R {
        use x86_64::instructions::tlb;
        use x86_64::registers::control_regs;

        let ret = {
            let backup = PhysicalAddress(control_regs::cr3().0 as usize);

            // map temporary_page to current p4 table
            let p4_table = unsafe {
                temporary_page.map_table_frame(backup.clone(), self)
            };

            // overwrite recursive mapping
            self.p4_mut()[511].set(
                table.p4_frame.clone(),
                EntryFlags::PRESENT | EntryFlags::WRITABLE
            );

            tlb::flush_all();

            // execute f in the new context
            let ret = f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);

            tlb::flush_all();

            ret
        };

        unsafe {
            temporary_page.unmap(self);
        }

        ret
    }

    pub fn switch(&mut self, new_table: InactivePageMap) -> InactivePageMap {
        use x86_64::registers::control_regs;

        let old_table = InactivePageMap {
            p4_frame: PhysicalAddress(control_regs::cr3().0 as usize)
        };

        unsafe {
            control_regs::cr3_write(x86_64::PhysicalAddress(new_table.p4_frame.0 as u64));
        }

        old_table
    }
}

impl Deref for ActivePageMap {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageMap {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

pub struct InactivePageMap {
    p4_frame: PhysicalAddress,
}

impl InactivePageMap {
    pub fn new(
        frame: PhysicalAddress,
        active_table: &mut ActivePageMap,
        temporary_page: &mut TemporaryPage)
    -> InactivePageMap {
        {
            let table = unsafe {
                temporary_page.map_table_frame(frame.clone(), active_table)
            };

            table.zero();

            // Set up recursive mapping for table
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }

        unsafe {
            temporary_page.unmap(active_table);
        }

        InactivePageMap { p4_frame: frame }
    }
}


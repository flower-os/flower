// Taken from https://os.phil-opp.com/remap-the-kernel/
// Many thanks!

use super::*;
use core::ops::{Deref, DerefMut};
use core::ops::RangeInclusive;
use util::round_up_divide;
use core::ops::Range;
use alloc::vec::Vec;

pub struct Mapper {
    p4: Unique<PageTable<Level4>>,
}

impl Mapper {
    const unsafe fn new() -> Self {
        // The address points to the recursively mapped entry (511) in the P4 table, which we can
        // use to access the P4 table itself.
        //                sign ext  p4  p3  p2  p1  offset
        const P4: usize = 0o177_777_776_776_776_776_0000;

        Mapper {
            p4: Unique::new_unchecked(P4 as *mut _),
        }
    }

    fn p4(&self) -> &PageTable<Level4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe { self.p4.as_mut() }
    }

    /// Walks the page tables and translates this page into a physical address
    pub fn walk_page_table(&self, page: Page) -> Option<(PageTableEntry, PageSize)> {
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
                        if p2_entry.flags().contains(EntryFlags::PRESENT | EntryFlags::HUGE_PAGE) {
                            // Check that the address is 2MiB aligned
                            assert_eq!(
                                (start_frame.0 >> 12) % PAGE_TABLE_ENTRIES,
                                0,
                                "Adress is not 2MiB aligned!"
                            );
                            return Some((
                                *p2_entry,
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
            .and_then(|p1| {
                let p1_entry = p1[page.p1_index()];
                if p1_entry.flags().contains(EntryFlags::PRESENT) {
                    Some((p1_entry, PageSize::Kib4))
                } else {
                    None
                }
            })
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

            let mut p1 = match p2.next_table_create(page.p2_index()) {
                Some(p1) => p1,
                None => {
                    if p2[page.p2_index()].flags().contains(EntryFlags::HUGE_PAGE) {
                        panic!("No next p1 table - the area is mapped in 2mib pages")
                    } else {
                        panic!("No next p1 table (unknown reason)")
                    }
                }
            };


            // 4kib page
            p1[page.p1_index()].set(
                physical_address,
                flags | EntryFlags::PRESENT,
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
                // TODO
                trace!("freeing physmem"); // TODO
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

    /// Maps a range of higher half addresses as 4kib pages in the -2GiB higher "half", mapping
    /// them to their address minus `KERNEL_MAPPING_BEGIN`.
    pub unsafe fn higher_half_map_range(
        &mut self,
        addresses: Range<usize>,
        flags: EntryFlags,
        invplg: bool,
    ) {
        let frame_end = round_up_divide(addresses.end as u64, 4096) as usize;
        for frame_no in (addresses.start / 4096)..=frame_end  {
            let address = frame_no * 4096;

            self.map_to(
                Page::containing_address(address, PageSize::Kib4),
                PhysicalAddress((address - ::memory::KERNEL_MAPPING_BEGIN) as usize),
                flags,
                invplg,
            );
        }
    }

    pub unsafe fn map_page_range(&mut self, mapping: PageRangeMapping, invplg: bool, flags: EntryFlags) {
        let frames = mapping.start_frame..=mapping.start_frame + mapping.pages.size_hint().1.unwrap();

        for (frame_no, page_no) in frames.zip(mapping.pages) {
            let phys_address = frame_no * 4096;
            let virtual_address = page_no * 4096;

            self.map_to(
                Page::containing_address(virtual_address, PageSize::Kib4),
                PhysicalAddress(phys_address as usize),
                flags,
                invplg,
            );
        }
    }

}

/// A 4kib page range mapping -- represents a contigous area of 4kib pages mapped to a contigous
/// area of 4kib frames. However, this does not need to be an identity mapping, i.e there may be
/// an offset
pub struct PageRangeMapping {
    /// Range of page numbers
    pub pages: RangeInclusive<usize>,

    /// The start frame number
    pub start_frame: usize,
}

impl PageRangeMapping {
    pub fn new(start_page: Page, start_frame: usize, pages: usize) -> PageRangeMapping {
        assert_eq!(start_page.page_size(), Some(PageSize::Kib4), "Start page needs to be 4kib!");
        let page_number = start_page.start_address().unwrap() / 4096;

        PageRangeMapping {
            pages: page_number..=(page_number + pages),
            start_frame,
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
        active_table: &mut ActivePageMap
    ) -> VirtualAddress {
        let page_addr = self.page.start_address().expect("Temporary page requires size");
        assert!(
            active_table.walk_page_table(self.page).is_none(),
            "Temporary page {:?} at 0x{:x} is already mapped",
            self.page,
            page_addr,
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
            self.p4_mut()[510].set(
                table.p4_frame.clone(),
                EntryFlags::PRESENT | EntryFlags::WRITABLE //| EntryFlags::NO_EXECUTE // TODO
            );

            tlb::flush_all();

            // execute f in the new context
            let ret = f(self);

            // restore recursive mapping to original p4 table
            p4_table[510].set(
                backup,
              EntryFlags::PRESENT | EntryFlags::WRITABLE //| EntryFlags::NO_EXECUTE // TODO
            );

            tlb::flush_all();

            ret
        };

        unsafe {
            temporary_page.unmap(self);
        }

        ret
    }

    pub fn remap_range(
        &mut self,
        new_table: &mut InactivePageMap,
        temporary_page: &mut TemporaryPage,
        pages: RangeInclusive<usize>,
        flags: EntryFlags
    ) {
        let num_pages = pages.end() - pages.start();
        let mut frames = Vec::with_capacity(num_pages);
        for i in 0..=num_pages {
            let page = Page::containing_address(
                (i + pages.start()) * 4096,
                PageSize::Kib4
            );

            let entry = PAGE_TABLES.lock().walk_page_table(page).unwrap().0;
            frames.push(entry.physical_address().unwrap());
        }

        self.with_inactive_p4(new_table, temporary_page, |mapper| {
            for page_no in pages.clone() {
                let page = Page::containing_address(page_no * 4096, PageSize::Kib4);
                let phys_addr = frames[page_no - pages.start()];

                unsafe {
                    mapper.map_to(page, phys_addr, flags, false);
                }
            }
        });
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
            table[510].set(
                frame.clone(),
                EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE
            );
        }

        unsafe {
            temporary_page.unmap(active_table);
        }

        InactivePageMap { p4_frame: frame }
    }
}


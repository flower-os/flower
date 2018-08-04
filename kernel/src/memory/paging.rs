//! Various functions and structures to work with paging, page tables, and page table entries.
//! Thanks a __lot__ to [Phil Opp's paging blogpost](https://os.phil-opp.com/page-tables/).

use spin::Mutex;
use core::{marker::PhantomData, convert::From, ops::{Index, IndexMut}, ptr::Unique};
use super::physical_allocator::PHYSICAL_ALLOCATOR;

const PAGE_TABLE_ENTRIES: usize = 512;
pub static PAGE_TABLES: Mutex<PageMap> = Mutex::new(PageMap::new());

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct PhysicalAddress(pub usize);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct VirtualAddress(usize);

impl From<VirtualAddress> for PhysicalAddress {
    fn from(vaddr: VirtualAddress) -> Self {
        // Check that the address is valid. Since it's a logic error if it isn't valid, then panic.
        assert!(
            vaddr.0 < 0x0000_8000_0000_0000 || vaddr.0 >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}", vaddr.0,
        );

        let page = Page {
            number: vaddr.0 / 4096, // Still valid for 2mib page
            size: None, // ... but we don't know the size :(
        };

        let (start, size) = PAGE_TABLES.lock().walk_page_table(page).unwrap();

        PhysicalAddress(start.0 + (vaddr.0 % size.bytes()))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum PageSize {
    Kib4,
    Mib2,
}

impl PageSize {
    fn bytes(self) -> usize {
        use self::PageSize::*;

        match self {
            Kib4 => 4 * 1024,
            Mib2 => 2 * 1024 * 1024,
        }
    }
}

pub struct PageMap {
    p4: Unique<PageTable<Level4>>,
}

impl PageMap {
    const fn new() -> PageMap {
        PageMap {
            // The address points to the recursively mapped entry in the P4 table, which we can use
            // to access the P4 table itself.
            p4: unsafe { Unique::new_unchecked(0xffffffff_fffff000 as *mut _) },
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
                            assert_eq!(start_frame.0 >> 12 % PAGE_TABLE_ENTRIES, 0);
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

    pub fn map_to(&mut self, page: Page, physical_address: PhysicalAddress, flags: EntryFlags) {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index()).unwrap();
        let mut p2 = p3.next_table_create(page.p3_index()).unwrap();
        let mut p1 = p2.next_table_create(page.p2_index());

        if let Some(p1) = p1 {
            // 4kib huge page
            p1[page.p1_index()].set(physical_address, flags | self::EntryFlags::PRESENT);
        } else {
            // 2mib huge page
            p2[page.p2_index()].set(physical_address, flags | self::EntryFlags::PRESENT);
        }
    }

    pub fn map(&mut self, page: Page, flags: EntryFlags) {
        use core::ptr;

        let ptr = PHYSICAL_ALLOCATOR.allocate(0).expect("out of memory");
        let frame = PhysicalAddress(ptr as usize);
        self.map_to(page, frame, flags);

        // Zero the page
        unsafe {
            ptr::write_bytes(
                page.start_address().unwrap() as *mut u8,
                0,
                page.size.unwrap().bytes()
            );
        }
    }

    pub fn unmap(&mut self, page: Page) {
        use x86_64::instructions::tlb;

        assert!(self.walk_page_table(page).is_some());

        let mut p2 = self.p4_mut()
            .next_page_table_mut(page.p4_index()).expect("Unmap called on unmapped page!")
            .next_page_table_mut(page.p3_index()).expect("Unmap called on unmapped page!");

        let p1 = p2.next_page_table_mut(page.p2_index());

        if let Some(p1) = p1 {
            // 4kib page

            let frame = p1[page.p1_index()].physical_address().unwrap();
            p1[page.p1_index()].set_unused();

            // TODO free p1/p2/p3 tables if they are empty
            PHYSICAL_ALLOCATOR.deallocate(frame.0 as *const _, 0);
        } else {
            // Huge 2mib page

            let frame = p2[page.p2_index()].physical_address().unwrap();
            p2[page.p2_index()].set_unused();

            // TODO free p2/p3 tables if they are empty
            PHYSICAL_ALLOCATOR.deallocate(frame.0 as *const _, 0);
        }

        // Flush tlb
        tlb::flush(::x86_64::VirtualAddress(page.start_address().unwrap()));
    }
}

#[derive(Copy, Clone)]
pub struct Page {
    number: usize,
    /// Size of page. None when unknown.
    size: Option<PageSize>,
}

impl Page {
    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        self.number & 0o777
    }

    fn start_address(&self) -> Option<usize> {
        self.size.map(|size| self.number * size.bytes())
    }

    pub fn containing_address(addr: usize, size: PageSize) -> Page {
        Page { number: addr / size.bytes(), size: Some(size) }
    }
}

/// An entry in a page table
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(C)] // Just in case
pub struct PageTableEntry(u64);

impl PageTableEntry {
    #[cfg(not(test))]
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn physical_address(&self) -> Option<PhysicalAddress> {
        if self.flags().contains(self::EntryFlags::PRESENT) {
            // Mask out the flag bits
            Some(PhysicalAddress(self.0 as usize & 0x000fffff_fffff000))
        } else {
            None
        }
    }

    pub fn set(&mut self, physical_address: PhysicalAddress, flags: EntryFlags) {
        // Check that the physical address is 1) page aligned and 2) not larger than max
        assert_eq!(physical_address.0 & !0x000fffff_fffff000, 0);
        self.0 = (physical_address.0 as u64) | flags.bits();
    }
}

bitflags! {
    pub struct EntryFlags: u64 {
        /// Whether the page is present in memory
        const PRESENT = 1;
        /// Whether the page is writable or read only
        const WRITABLE = 1 << 1;
        /// Whether ring 3 processes can access this page -- in theory. As of meltdown, this bit is
        /// essentially useless, except on possibly newer CPUs with fixes in place
        const USER_ACCESSIBLE = 1 << 2;
        /// If this bit is set, writes to this page go directly to memory
        const WRITE_DIRECT = 1 << 3;
        /// Do not use cache for this page
        const NO_CACHE = 1 << 4;
        /// Set by the CPU when this page has been accessed
        const ACCESSED = 1 << 5;
        /// Set by the CPU when this page is written to
        const DIRTY = 1 << 6;
        /// Whether this page is a huge page. 0 in P1 and P4, but sets this as a 1GiB page in P3
        /// and a 2MiB page in P2
        const HUGE_PAGE = 1 << 7;
        /// If set, this page will not be flushed in the TLB. PGE bit in CR4 must be set.
        const GLOBAL = 1 << 8;
        /// Do not allow executing code from this page. NXE bit in EFER must be set.
        const NO_EXECUTE = 1 << 63;
    }
}

/// A trait that indicates a type represents a page table level
pub trait TableLevel {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

/// A trait that indicates a type represents a page table level that is not P1
pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;

    const CAN_BE_HUGE: bool = false;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
    const CAN_BE_HUGE: bool = true;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
    const CAN_BE_HUGE: bool = true;
}

/// A page table consisting of 512 entries ([PageTableEntry]).
pub struct PageTable<L: TableLevel> {
    entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
    _level: PhantomData<L>,
}

impl<L: TableLevel> PageTable<L> {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    fn next_table_addr(&self, index: usize) -> Option<usize>
        where L: HierarchicalLevel
    {
        let entry_flags = self[index].flags();
        if entry_flags.contains(self::EntryFlags::PRESENT) && !entry_flags.contains(self::EntryFlags::HUGE_PAGE) {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    fn next_page_table(&self, index: usize) -> Option<&PageTable<L::NextLevel>>
        where L: HierarchicalLevel
    {
        unsafe {
            self.next_table_addr(index)
                .map(|addr| &*(addr as *const _))
        }
    }

    fn next_page_table_mut(&mut self, index: usize) -> Option<&mut PageTable<L::NextLevel>>
        where L: HierarchicalLevel
    {
        unsafe {
            self.next_table_addr(index)
                .map(|addr| &mut *(addr as *mut _))
        }
    }


    pub fn next_table_create(&mut self, index: usize) -> Option<&mut PageTable<L::NextLevel>>
        where L: HierarchicalLevel
    {
        if self.next_page_table(index).is_none() {
            if self.entries[index].flags().contains(self::EntryFlags::HUGE_PAGE){
                assert!(L::CAN_BE_HUGE, "Page has huge bit but cannot be huge!");
            } else {
                let ptr = PHYSICAL_ALLOCATOR.allocate(0).expect("No physical frames available!");
                let frame = PhysicalAddress(ptr as usize);

                self.entries[index].set(frame, self::EntryFlags::PRESENT | self::EntryFlags::WRITABLE);
                self.next_page_table_mut(index).unwrap().zero();
            }
        }

        self.next_page_table_mut(index)
    }
}

impl<L: TableLevel> Index<usize> for PageTable<L> {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }
}

impl<L: TableLevel> IndexMut<usize> for PageTable<L> {
    fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

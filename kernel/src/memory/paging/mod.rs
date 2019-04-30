//! Various functions and structures to work with paging, page tables, and page table entries.
//! Thanks a __lot__ to [Phil Opp's paging blogpost](https://os.phil-opp.com/page-tables/).

pub mod remap;
mod page_map;
pub use self::page_map::*;

use core::{marker::PhantomData, ptr::Unique};
use core::ops::{Add, Sub, Index, IndexMut};
use spin::Mutex;
use super::physical_allocator::PHYSICAL_ALLOCATOR;
use x86_64::instructions::tlb;

const PAGE_TABLE_ENTRIES: usize = 512;
pub static ACTIVE_PAGE_TABLES: Mutex<ActivePageMap> = Mutex::new(unsafe { ActivePageMap::new() });

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct PhysicalAddress(pub usize);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct VirtualAddress(pub usize);

/// The size of a page. Distinct from `memory::PageSize` in that it only enumerates page sizes
/// supported by the paging module at this time.
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

#[derive(Debug, Copy, Clone)]
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

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn start_address(&self) -> Option<usize> {
        self.size.map(|size| self.number * size.bytes())
    }

    pub fn page_size(&self) -> Option<PageSize> {
        self.size
    }

    pub fn containing_address(addr: usize, size: PageSize) -> Page {
        Page { number: addr / size.bytes(), size: Some(size) }
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, other: usize) -> Page {
        Page {
            number: self.number + other,
            size: self.size
        }
    }
}

impl Sub<usize> for Page {
    type Output = Page;

    fn sub(self, other: usize) -> Page {
        Page {
            number: self.number - other,
            size: self.size
        }
    }
}

/// An entry in a page table
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)] // Just in case
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn physical_address(&self) -> Option<PhysicalAddress> {
        if self.flags().contains(self::EntryFlags::PRESENT) {
            Some(PhysicalAddress(self.0 as usize & 0x000FFFFF_FFFFF000)) // Mask out the flag bits
        } else {
            None
        }
    }

    pub fn set(&mut self, physical_address: PhysicalAddress, flags: EntryFlags) {
        // Check that the physical address is page aligned
        assert_eq!(
            physical_address.0 & 0xFFF,
            0,
            "Physical address 0x{:x} not page aligned!",
            physical_address.0,
        );

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
        /// essentially useless for preventing attacks, except on possibly newer CPUs with fixes
        /// in place.
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
        const GLOBAL = 1 << 8; // TODO map kernel pages as global?
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
            Some((0xFFFF << 48) | (table_address << 9) | (index << 12))
            // HEADS UP ^. This first mask would change if the p4 table were recursively mapped to
            // an entry in the 0 sign extended half of the address space. BEWARE!
        } else {
            None
        }
    }

    pub fn next_page_table(&self, index: usize) -> Option<&PageTable<L::NextLevel>>
        where L: HierarchicalLevel
    {
        unsafe {
            self.next_table_addr(index)
                .map(|addr| &*(addr as *const _))
        }
    }

    pub fn next_page_table_mut(&mut self, index: usize) -> Option<&mut PageTable<L::NextLevel>>
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

                self.entries[index].set(
                    frame,
                    self::EntryFlags::PRESENT |
                        self::EntryFlags::WRITABLE |
                        self::EntryFlags::USER_ACCESSIBLE
                );
                self.next_page_table_mut(index).expect("No next table!").zero();
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

pub fn new_process_page_tables() -> InactivePageMap {
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

    // Drop this lock so that the RAII guarded temporary page can be destroyed
    drop(active_table);

    new_table
}

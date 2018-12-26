//! All things to do with memory

use core::convert::From;
use core::{iter, mem};
use arrayvec::ArrayVec;

#[macro_use]
mod buddy_allocator;
pub mod paging;
pub mod heap;
pub mod bootstrap_heap;
pub mod physical_allocator;

use core::ops::{Range, Deref};
use core::ptr::NonNull;
use multiboot2::{BootInformation, MemoryMapTag};
use self::physical_allocator::{PHYSICAL_ALLOCATOR, BLOCKS_IN_TREE};
use self::buddy_allocator::Block;
use self::bootstrap_heap::BOOTSTRAP_HEAP;
use crate::util;

/// Represents the size of a page.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum PageSize {
    Kib4,
    Mib2,
    Gib1,
}

impl From<PageSize> for usize {
    fn from(size: PageSize) -> Self {
        use self::PageSize::*;
        match size {
            Kib4 => 4 * 1024,
            Mib2 => 2 * 1024 * 1024,
            Gib1 => 1024 * 1024 * 1024,
        }
    }
}

pub unsafe fn map_physical_region<T>(
    physical_address: usize,
    size: usize,
    mutable: bool
) -> PhysicalMapping<T> {
    let frames = util::round_up_divide(size as u64, 4096) as usize;
    let physical_begin_frame = physical_address / 4096;

    let alloc_ptr = crate::HEAP.alloc_specific(physical_begin_frame, frames) as usize;

    if alloc_ptr == 0 {
        panic!("Ran out of heap memory!");
    }

    let obj_ptr = alloc_ptr + physical_address - (physical_begin_frame * 4096);

    PhysicalMapping {
        physical_start: physical_begin_frame * 4096,
        // alloc_ptr is zero if there is no more heap memory available
        virtual_start: NonNull::new(obj_ptr as *mut T)
            .expect("Ran out of heap memory!"),
        mapped_length: frames * 4096,
        mutable
    }
}

pub unsafe fn map_physical_type<T>(physical_address: usize, mutable: bool) -> PhysicalMapping<T> {
    map_physical_region(physical_address, mem::size_of::<T>(), mutable)
}

pub struct PhysicalMapping<T> {
    physical_start: usize,
    virtual_start: NonNull<T>,
    mapped_length: usize,
    mutable: bool,
}

impl<T> PhysicalMapping<T> {
    /// Returns a mutable reference to the data if this mapping is mutable and returns None if not
    /// mutable.
    pub fn deref_mut(&mut self) -> Option<&mut T> {
        if self.mutable {
            Some(unsafe { self.virtual_start.as_mut() })
        } else {
            None
        }
    }
}

impl<T> Drop for PhysicalMapping<T> {
    fn drop(&mut self) {
        let obj_addr = self.virtual_start.as_ptr() as *mut T as usize;

        // Clear lower page offset bits
        let page_begin = obj_addr & !0xFFF;

        unsafe {
            crate::HEAP.dealloc_specific(
                page_begin as *mut u8,
                self.mapped_length / 4096,
            );
        }
    }
}

impl<T> Deref for PhysicalMapping<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.virtual_start.as_ref() }
    }
}

impl<T> Into<::acpi::PhysicalMapping<T>> for PhysicalMapping<T> {
    fn into(self) -> ::acpi::PhysicalMapping<T> {
        let mapping = ::acpi::PhysicalMapping {
            physical_start: self.physical_start,
            virtual_start: self.virtual_start,
            region_length: self.mapped_length,
            mapped_length: self.mapped_length,
        };
        mem::forget(self);
        mapping
    }
}

pub fn init_memory(mb_info: &BootInformation, guard_page_addr: usize) {
    info!("mem: initialising");
    let memory_map = mb_info.memory_map_tag()
        .expect("Expected a multiboot2 memory map tag, but it is not present!");

    print_memory_info(memory_map);

    trace!("mem: setting up guard page");
    setup_guard_page(guard_page_addr);

    debug!("mem: initialising bootstrap heap");
    setup_bootstrap_heap(mb_info);

    debug!("mem: initialising pmm (1/2)");
    let (gibbibytes, usable) = setup_physical_allocator_prelim(mb_info);

    debug!("mem: setting up kernel heap");
    crate::HEAP.init();

    debug!("mem: initialising pmm (2/2)");
    setup_physical_allocator_rest(gibbibytes, usable.iter());

    info!("mem: initialised")
}

fn print_memory_info(memory_map: &MemoryMapTag) {
    trace!("mem: Usable memory areas: ");

    // For when log_level != debug | trace
    #[allow(unused_variables)]
        for area in memory_map.memory_areas() {
        trace!(" 0x{:x} to 0x{:x}",
               area.start_address(), area.end_address());
    }

    // Calculate how many GiBs are available
    let bytes_available: u64 = memory_map.memory_areas()
        .map(|area| area.end_address() - area.start_address())
        .sum();

    let gibbibytes_available = bytes_available as f64 / (1 << 30) as f64;
    info!("{:.3} GiB of RAM available", gibbibytes_available);
}

fn setup_bootstrap_heap(mb_info: &BootInformation) {
    use core::cmp;

    // MB info struct could be higher than kernel, so take max
    let kernel_end = kernel_area(mb_info).end;
    let mb_info_end = mb_info.end_address();
    let end_address = cmp::max(kernel_end, mb_info_end) as *const u8;

    let end_address = unsafe {
        end_address.offset(
            end_address.align_offset(mem::align_of::<[Block; BLOCKS_IN_TREE]>()) as isize,
        )
    };

    let heap_start = end_address;
    unsafe { BOOTSTRAP_HEAP.init_unchecked(heap_start as usize); }
}

fn setup_physical_allocator_prelim(
    mb_info: &BootInformation
) -> (u8, ArrayVec<[Range<usize>; 256]>) {
    let memory_map = mb_info.memory_map_tag()
        .expect("Expected a multiboot2 memory map tag, but it is not present!");

    let highest_address = memory_map.memory_areas()
        .map(|area| area.end_address() - 1)
        .max()
        .expect("No usable physical memory available!");

    // Do round-up division by 2^30 = 1GiB in bytes
    let trees = util::round_up_divide(highest_address as u64, 1 << 30) as u8;
    trace!("Allocating {} trees", trees);

    let kernel_area = kernel_area(mb_info).start..BOOTSTRAP_HEAP.end() + 1;

    // Calculate the usable memory areas by using the MB2 memory map but excluding kernel areas
    let usable_areas = memory_map
        .memory_areas()
        .map(|area| (area.start_address() as usize, area.end_address() as usize))
        .map(|(start, end)| start..end)
        .flat_map(move |area| { // Remove kernel areas
            // HACK: arrays iterate with moving weirdly
            // Also, filter map to remove `None`s
            let [first, second] = range_sub(&area, &kernel_area);
            iter::once(first).chain(iter::once(second)).filter_map(|i| i)
        })
        .flat_map(move |area| { // Remove areas below 1mib
            // HACK: arrays iterate with moving weirdly
            // Also, filter map to remove `None`s
            let [first, second] = range_sub(&area, &(0..1024 * 1024));
            iter::once(first).chain(iter::once(second)).filter_map(|i| i)
        })
        .collect::<ArrayVec<[_; 256]>>(); // Collect here into a large ArrayVec for performance

    PHYSICAL_ALLOCATOR.init_prelim(usable_areas.iter());

    (trees, usable_areas)
}

fn setup_physical_allocator_rest<'a, I>(gibbibytes: u8, usable_areas: I)
    where I: Iterator<Item=&'a Range<usize>> + Clone + 'a
{
    PHYSICAL_ALLOCATOR.init_rest(
        gibbibytes,
        usable_areas,
    );
}

fn setup_guard_page(addr: usize) {
    use self::paging::*;

    PAGE_TABLES.lock().unmap(Page::containing_address(addr, PageSize::Kib4), false);
}

fn kernel_area(mb_info: &BootInformation) -> Range<usize> {
    let elf_sections = mb_info.elf_sections_tag()
        .expect("Expected a multiboot2 elf sections tag, but it is not present!");
    let modules = mb_info.module_tags();

    let used_areas = elf_sections.sections()
        .map(|section| section.start_address()..section.end_address() + 1)
        .chain(modules.map(|module|
            module.start_address() as u64..module.end_address() as u64
        ));

    let begin = used_areas.clone().map(|range| range.start).min().unwrap() as usize;
    let end = (used_areas.map(|range| range.end).max().unwrap() + 1) as usize;

    begin..end
}

/// Subtracts a range from another one
fn range_sub<T>(
    main: &Range<T>,
    sub: &Range<T>,
) -> [Option<Range<T>>; 2]
    where T: Ord + Copy,
{
    let hole_start = if sub.start >= main.start && sub.start < main.end {
        Some(sub.start)
    } else if sub.end >= main.start && sub.end <= main.start {
        Some(main.start)
    } else {
        None
    };

    let hole_end = if main.end > sub.end && hole_start.is_some() {
        Some(sub.end)
    } else if hole_start.is_some() {
        Some(main.end)
    } else {
        None
    };

    let hole = match (hole_start, hole_end) {
        (Some(start), Some(end)) => Some(start..end),
        _ => None,
    };

    if let Some(hole) = hole {
        let lower_half = if main.start != hole.start {
            Some(main.start..hole.start)
        } else {
            None
        };

        let higher_half = if main.end != hole.end {
            Some(hole.end..main.end)
        } else {
            None
        };

        [lower_half, higher_half]
    } else {
        [Some(main.clone()), None]
    }
}

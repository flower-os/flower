use core::convert::From;
use core::{iter, mem};

#[macro_use]
mod buddy_allocator;
mod paging;
pub mod heap;
pub mod bootstrap_heap;
pub mod physical_allocator;

use core::ops::Range;
use multiboot2::{BootInformation, MemoryMapTag};
use self::physical_allocator::{PHYSICAL_ALLOCATOR, BLOCKS_IN_TREE};
use self::buddy_allocator::Block;
use self::bootstrap_heap::BOOTSTRAP_HEAP;

/// Represents the size of a page
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

pub fn init_memory(mb_info: &BootInformation, guard_page_addr: usize) {
    info!("mem: initialising");
    let memory_map = mb_info.memory_map_tag()
        .expect("Expected a multiboot2 memory map tag, but it is not present!");

    print_memory_info(memory_map);
    debug!("mem: initialising bootstrap heap");
    setup_bootstrap_heap(mb_info);
    debug!("mem: initialising pmm (1/2)");
    let (gibbibytes, usable) = setup_physical_allocator_prelim(mb_info);
    trace!("mem: setting up guard page");
    setup_guard_page(guard_page_addr);
    debug!("mem: setting up kernel heap");
    ::HEAP.init();
    debug!("mem: initialising pmm (2/2)");
    setup_physical_allocator_rest(gibbibytes, usable);
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
    let bytes_available: usize = memory_map.memory_areas()
        .map(|area| area.start_address() + area.end_address())
        .sum();

    let gibbibytes_available  = bytes_available as f64 / (1 << 30) as f64;
    info!("{:.3} GiB of RAM available", gibbibytes_available);
}

fn setup_bootstrap_heap(mb_info: &BootInformation) {
    use core::cmp;

    // MB info struct could be higher than kernel, so take max
    let kernel_end = kernel_area(mb_info).end;
    let mb_info_end =  mb_info.end_address();
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
) -> (u8, impl Iterator<Item=Range<usize>> + Clone) {
    let memory_map = mb_info.memory_map_tag()
        .expect("Expected a multiboot2 memory map tag, but it is not present!");

    let highest_address = memory_map.memory_areas()
        .map(|area| area.end_address() - 1)
        .max()
        .expect("No usable physical memory available!");

    // Do round-up division by 2^30 = 1GiB in bytes
    let trees = ((highest_address + (1 << 30) - 1) / (1 << 30)) as u8;
    trace!("Allocating {} trees", trees);

    let kernel_area = kernel_area(mb_info).start..BOOTSTRAP_HEAP.end() + 1;

    // Calculate the usable memory areas by using the MB2 memory map but excluding kernel areas
    let usable_areas = memory_map
        .memory_areas()
        .map(|area| (area.start_address() as usize, area.end_address() as usize))
        .map(|(start, end)| start..end)
        .flat_map(move |area| {
            // HACK: arrays iterate with moving weirdly
            // Also, filter map to remove `None`s
            let [first, second] = range_sub(&area, &kernel_area);
            iter::once(first).chain(iter::once(second)).filter_map(|i| i)
        });

    PHYSICAL_ALLOCATOR.init_prelim(
        trees,
        usable_areas.clone(),
    );

    (trees, usable_areas)
}

fn setup_physical_allocator_rest<I>(gibbibytes: u8, usable_areas: I)
    where I: Iterator<Item=Range<usize>> + Clone
{
    PHYSICAL_ALLOCATOR.init_rest(
        gibbibytes,
        usable_areas,
    );
}

fn setup_guard_page(addr: usize) {
    use self::paging::*;

    PAGE_TABLES.lock().unmap(Page::containing_address(addr, PageSize::Kib4));
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
    sub: &Range<T>
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

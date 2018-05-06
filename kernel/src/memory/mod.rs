use core::convert::From;

#[macro_use]
mod buddy_allocator;
pub mod bootstrap_heap;
pub mod physical_allocator;

use multiboot2::{BootInformation, MemoryMapTag};

/// The size of a physical frame
pub const FRAME_SIZE: usize = 4096;

/// A structure representing a physical address
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PhysicalAddress(pub usize);

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

pub fn init_memory(mb_info: &BootInformation) {
    use core::mem;
    use self::physical_allocator::{PHYSICAL_ALLOCATOR, BLOCKS_IN_TREE};
    use self::buddy_allocator::Block;

    let memory_map = mb_info.memory_map_tag()
        .expect("Expected a multiboot2 memory map tag, but it is not present!");

    print_usable_areas(memory_map);

    // Set up bootstrap heap
    let end_address = mb_info.end_address() as *const u8;
    let end_address = unsafe { end_address.offset(
        end_address.align_offset(mem::align_of::<[Block; BLOCKS_IN_TREE]>()) as isize
    )};
    let heap_start = end_address;
    unsafe { bootstrap_heap::BOOTSTRAP_HEAP.init_unchecked(heap_start as usize); }

    let highest_address = memory_map.memory_areas()
        .map(|area| area.end_address() - 1)
        .max()
        .expect("No usable physical memory available!");

    // Do round-up division by 2^30 = 1GiB in bytes
    let trees = ((highest_address + (1 << 30) - 1) / (1 << 30)) as u8;

    let bytes_available: usize = memory_map.memory_areas()
        .map(|area| area.end_address() - area.start_address() - 1)
        .sum();
    let gibbibytes_available  = bytes_available as f64 / (1 << 30) as f64;

    debug!("Allocating {} trees", trees);
    info!("{:.2} GiB of RAM available", gibbibytes_available);

    // Set up physical frame allocator
    PHYSICAL_ALLOCATOR.init(
        trees,
        memory_map.memory_areas()
            .map(|area| (area.start_address() as usize, area.end_address() as usize))
            .map(|(start, end)| start..=(end - 1))
    );

    let mut address = 0x0 as *const _;

    // TODO just do this for fun, but remove later
    for _ in 0..2 {
        address = PHYSICAL_ALLOCATOR.allocate(0).unwrap();
        debug!("Allocated {:?}", address);
    }

    PHYSICAL_ALLOCATOR.deallocate(address, 0);
    debug!("Freed {:?}", address);

    debug!("Allocated {:?}", PHYSICAL_ALLOCATOR.allocate(0).unwrap());

}

fn print_usable_areas(memory_map: &MemoryMapTag) {
    debug!("mem: Usable memory areas: ");
    for area in memory_map.memory_areas() {
        debug!(" 0x{:x} to 0x{:x}",
               area.start_address(), area.end_address());
    }
}

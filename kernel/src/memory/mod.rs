use core::convert::From;

pub mod bootstrap_allocator;
pub mod buddy_allocator;

/// The size of a physical frame
pub const FRAME_SIZE: usize = 4 * 4096;

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

///! A modified buddy bitmap allocator. Written originally in
/// [buddy allocator workshop](https://github.com/Restioson/buddy-allocator-workshop).
use core::{mem, ptr, ops::RangeInclusive};
use spin::{RwLock, Mutex};

/// Number of orders.
const LEVEL_COUNT: u8 = 19;
/// The base order size. All orders are in context of this -- i.e the size of a block of order `k`
/// is `2^(k + MIN_ORDER)`, not `2^k`.
const BASE_ORDER: u8 = 12;

/// The physical frame allocator. Requires the bootstrap heap to be initialized, or else the
/// initializer will panic.
pub static PHYSICAL_ALLOCATOR: PhysicalAllocator<'static> = PhysicalAllocator {
    trees: RwLock::new(None),
};

buddy_allocator_bitmap_tree!(LEVEL_COUNT = 19, BASE_ORDER = 12);

pub struct PhysicalAllocator<'a> {
    // Max 256GiB
    trees: RwLock<Option<
        [Option<Mutex<Tree<'a>>>; 256]
    >>,
}

impl<'a> PhysicalAllocator<'a> {
    /// Create a new, initialized allocator
    #[cfg(test)]
    fn new(gibbibytes: u8, holes: &[RangeInclusive<usize>]) -> Self{
        let allocator = PhysicalAllocator {
            trees: RwLock::new(None),
        };

        allocator.init(gibbibytes, holes);

        allocator
    }

    /// Initialize the allocator. Panics if already initialized.
    pub fn init(&self, gibbibytes: u8, holes: &[RangeInclusive<usize>]) {
        if let Some(_) = *self.trees.read() {
            panic!("PhysicalAllocator already initialized!");
        }

        let mut trees: [Option<Mutex<Tree<'a>>>; 256] = unsafe { mem::uninitialized() };

        for (i, slot) in trees.iter_mut().enumerate() {
            if i > gibbibytes as usize {
                unsafe { ptr::write(slot as *mut _, None) };
            } else {
                unsafe {
                    ptr::write(slot as *mut _, Some(Mutex::new(Tree::new(&holes[..]))));
                }
            }
        }

        *self.trees.write() = Some(trees);
    }

    /// Allocate a frame of order `order`. Panics if not initialized.
    pub fn allocate(&self, order: u8) -> Option<*const u8> {
        #[derive(Eq, PartialEq, Copy, Clone)]
        enum TryState {
            Tried,
            WasInUse,
            Untried,
        }

        let mut tried = [TryState::Untried; 256];

        // Try every tree. If it's locked, come back to it later.
        loop {
            let index = tried.iter()
                .position(|i| *i == TryState::Untried)
                .or_else(
                    || tried.iter().position(|i| *i == TryState::WasInUse)
                )?;

            let lock = self.trees.read();
            let trees = lock.as_ref().unwrap();

            if let Some(ref tree) = trees[index] {
                if let Some(ref mut tree) = tree.try_lock() {
                    match tree.allocate(order) {
                        Some(b) => return Some(
                            (b as usize + (index * (1 << MAX_ORDER + BASE_ORDER))) as *const u8
                        ),
                        None => tried[index] = TryState::Tried,
                    }
                } else {
                    tried[index] = TryState::WasInUse;
                }
            } else {
                tried[index] = TryState::Tried;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alloc_physical_allocator() {
        let allocator = PhysicalAllocator::new(2, &[]);

        assert_eq!(allocator.allocate(0).unwrap(), 0x0 as *const u8);

        let lock = allocator.trees.read();
        let _tree_lock = lock.as_ref().unwrap()[0] // Get trees array
            .as_ref().unwrap().lock(); // Lock block

        assert_eq!(allocator.allocate(0).unwrap(), 2usize.pow((MAX_ORDER + BASE_ORDER) as u32) as *const u8);
    }
}

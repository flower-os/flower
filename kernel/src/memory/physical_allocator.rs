///! A modified buddy bitmap allocator. Written originally in
/// [buddy allocator workshop](https://github.com/Restioson/buddy-allocator-workshop).
use core::{mem, ptr, ops::Range};
#[cfg(test)]
use std::boxed::Box;
use spin::{Mutex, Once};
use super::bootstrap_heap::BootstrapHeapBox;

/// Number of orders.
const LEVEL_COUNT: u8 = 19;
/// The base order size. All orders are in context of this -- i.e the size of a block of order `k`
/// is `2^(k + MIN_ORDER)`, not `2^k`.
const BASE_ORDER: u8 = 12;

/// The physical frame allocator. Requires the bootstrap heap to be initialized, or else the
/// initializer will panic.
pub static PHYSICAL_ALLOCATOR: PhysicalAllocator<'static> = PhysicalAllocator {
    trees: Once::new(),
    #[cfg(test)]
    _phantom: ::core::marker::PhantomData,
};

// Panics from `buddy_allocator.rs` will say they're from here. Go there instead.
buddy_allocator_bitmap_tree!(LEVEL_COUNT = LEVEL_COUNT, BASE_ORDER = BASE_ORDER);

impl<'a> Tree<BootstrapHeapBox<'a, [Block; BLOCKS_IN_TREE]>> {
    /// Creates a new tree. Panics if the bootstrap allocator has not been initialized or
    /// does not have enough memory. Takes a list of usable memory areas, **local to this
    /// tree.**
    #[cfg(not(test))]
    pub fn new_bootstrap_heap<I>(usable: I) -> Self
        where I: Iterator<Item=Range<usize>> + Clone,
    {
        let mut flat_blocks: BootstrapHeapBox<[Block; BLOCKS_IN_TREE]> = unsafe {
            super::bootstrap_heap::BOOTSTRAP_HEAP.allocate_zeroed()
            .expect("No heap memory available!")
        };
        Self::new(usable, flat_blocks)
    }
}

pub struct PhysicalAllocator<'a> {
    // Max 256GiB
    #[cfg(not(test))]
    trees: Once<[Option<Mutex<Tree<BootstrapHeapBox<'a, [Block; BLOCKS_IN_TREE]>>>>; 256]>,
    #[cfg(test)]
    trees: Once<[Option<Mutex<Tree<Box<[Block; BLOCKS_IN_TREE]>>>>; 256]>,
    #[cfg(test)]
    _phantom: ::core::marker::PhantomData<&'a ()>,
}

impl<'a> PhysicalAllocator<'a> {
    /// Create a new, initialized allocator
    #[cfg(test)]
    fn new<I>(gibbibytes: u8, usable: I) -> Self
        where I: Iterator<Item=Range<usize>> + Clone
    {
        let allocator = PhysicalAllocator {
            trees: Once::new(),
            _phantom: ::core::marker::PhantomData,
        };

        allocator.init(gibbibytes, usable);

        allocator
    }

    /// Initialize the allocator.
    pub fn init<I>(&self, gibbibytes: u8, usable: I)
        where I: Iterator<Item=Range<usize>> + Clone
    {
        self.trees.call_once(|| {
            #[cfg(not(test))]
            let mut trees: [
                Option<Mutex<Tree<BootstrapHeapBox<'a, [Block; BLOCKS_IN_TREE]>>>>; 256
            ] = unsafe { mem::uninitialized() };
            #[cfg(test)]
            let mut trees: [
                Option<Mutex<Tree<Box<[Block; BLOCKS_IN_TREE]>>>>; 256
            ] = unsafe { mem::uninitialized() };

            for (i, slot) in trees.iter_mut().enumerate() {
                if i >= gibbibytes as usize {
                    unsafe { ptr::write(slot as *mut _, None) };
                } else {
                    // Filter out addresses that apply to this tree and make them local to it
                    let usable = (&usable).clone()
                        .filter_map(|range| {
                            let gib = (i << 30)..((i + 1 << 30) + 1);

                            // If the range covers any portion of the GiB
                            if !(range.start > gib.end) && !(range.end < gib.start) {
                                let end = range.end - gib.start;
                                let begin = if range.start >= gib.start {
                                    range.start - gib.start // Begin is within this GiB
                                } else {
                                    0 // Begin is earlier than this GiB
                                };

                                Some(begin..end)
                            } else {
                                None
                            }
                        });

                    unsafe {
                        #[cfg(not(test))]
                        ptr::write(
                            slot as *mut _,
                            Some(Mutex::new(Tree::new_bootstrap_heap(usable)))
                        );

                        #[cfg(test)]
                        ptr::write(
                            slot as *mut _,
                            Some(Mutex::new(Tree::new(usable, box mem::uninitialized())))
                        )
                    }
                }
            }

            trees
        });
    }

    /// Allocate a frame of order `order`. Panics if not initialized. Does __not__ zero the memory.
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

            let trees = self.trees.wait().unwrap();

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

    /// Deallocate the block of `order` at `ptr`. Panics if not initialized, if block is free, or if
    /// block is out of bounds of the # of GiB available.
    pub fn deallocate(&self, ptr: *const u8, order: u8) {
        let tree = (ptr as usize) >> (LEVEL_COUNT - 1 + BASE_ORDER);
        let local_ptr = (ptr as usize % (1 << LEVEL_COUNT - 1 + BASE_ORDER)) as *const u8;

        let trees = self.trees.wait().unwrap();
        let mut tree = trees[tree].as_ref().unwrap().lock();

        tree.deallocate(local_ptr, order);
    }
}

#[cfg(test)]
mod test {
    use core::iter;
    use super::*;

    #[test]
    fn test_alloc_physical_allocator() {
        let allocator = PhysicalAllocator::new(
            2,
            iter::once(0..(2 << MAX_ORDER + BASE_ORDER) + 1),
        );

        assert_eq!(allocator.allocate(0).unwrap(), 0x0 as *const u8);

        let trees = allocator.trees.wait().unwrap();
        let _tree_lock = trees[0].as_ref().unwrap().lock();

        assert_eq!(allocator.allocate(0).unwrap(), 2usize.pow((MAX_ORDER + BASE_ORDER) as u32) as *const u8);
    }

    #[test]
    fn test_dealloc_physical_allocator() {
        let allocator = PhysicalAllocator::new(
            2,
             iter::once(0..(2 << 30) + 1),
        );

        allocator.allocate(0).unwrap();
        allocator.deallocate(0x0 as *const u8, 0);
        assert_eq!(allocator.allocate(5).unwrap(), 0x0 as *const u8);
    }
}

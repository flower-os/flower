///! A modified buddy bitmap allocator. Written originally in
/// [buddy allocator workshop](https://github.com/Restioson/buddy-allocator-workshop).
use core::{mem, ptr, ops::{Range, Deref, DerefMut}};
#[cfg(test)]
use std::boxed::Box;
#[cfg(not(test))]
use alloc::boxed::Box;
use spin::{Mutex, Once};
use super::bootstrap_heap::{BootstrapHeapBox, BOOTSTRAP_HEAP};

/// Number of orders.
const LEVEL_COUNT: u8 = 19;
/// The base order size. All orders are in context of this -- i.e the size of a block of order `k`
/// is `2^(k + MIN_ORDER)`, not `2^k`.
const BASE_ORDER: u8 = 12;

/// The physical frame allocator. Requires the bootstrap heap to be initialized, or else the
/// initializer will panic.
pub static PHYSICAL_ALLOCATOR: PhysicalAllocator<'static> = PhysicalAllocator {
    trees: Once::new(),
};

// Panics from `buddy_allocator.rs` will say they're from here. Go there instead.
buddy_allocator_bitmap_tree!(LEVEL_COUNT = LEVEL_COUNT, BASE_ORDER = BASE_ORDER);

pub struct PhysicalAllocator<'a> {
    // Max 256GiB
    trees: Once<[Mutex<Option<Tree<TreeBox<'a>>>>; 256]>,
}

impl<'a> PhysicalAllocator<'a> {
    /// Create a new, initialized allocator
    #[cfg(test)]
    fn new<I>(gibbibytes: u8, usable: I) -> Self
        where I: Iterator<Item=Range<usize>> + Clone
    {
        let allocator = PhysicalAllocator {
            trees: Once::new(),
        };

        allocator.init_prelim(gibbibytes, usable.clone());
        allocator.init_rest(gibbibytes, usable);

        allocator
    }

    /// Initialize the allocator's first 8 gibbibytes. The PMM has a two stage init -- in the first
    /// stage, the first 8 GiBs are set up, using the bootstrap heap. This is enough to set up the
    /// main kernel heap. In the second stage, the rest of the GiBs are set up, using the kernel
    /// heap.
    pub fn init_prelim<I>(&self, gibbibytes: u8, usable: I)
        where I: Iterator<Item=Range<usize>> + Clone
    {
        self.trees.call_once(|| {
            let mut trees: [Mutex<Option<Tree<TreeBox<'a>>>>; 256] = unsafe {
                mem::uninitialized()
            };

            for (i, slot) in trees.iter_mut().enumerate() {
                if i >= gibbibytes as usize || i >= 8 {
                    unsafe { ptr::write(slot as *mut _, Mutex::new(None)) };
                } else {
                    let usable = Self::localize(i as u8, usable.clone());

                    unsafe {
                        #[cfg(not(test))]
                        let tree = Tree::new(
                            usable,
                            TreeBox::Bootstrap(
                                BOOTSTRAP_HEAP.allocate_zeroed()
                                    .expect("Ran out of bootstrap heap memory!")
                            )
                        );

                        #[cfg(test)]
                        let tree = Tree::new(
                            usable,
                            TreeBox::Heap(box mem::uninitialized()),
                        );

                        ptr::write(slot as *mut _, Mutex::new(Some(tree)));
                    }
                }
            }

            trees
        });
    }

    /// Initialise the rest of the allocator's gibbibytes. See [PhysicalAllocator.init_prelim].
    pub fn init_rest<I>(&self, gibbibytes: u8, usable: I)
        where I: Iterator<Item=Range<usize>> + Clone
    {
        let trees = self.trees.wait().unwrap();

        for i in 8..=gibbibytes {
            let usable = Self::localize(i as u8, usable.clone());

            unsafe {
                let tree = Tree::new(usable, TreeBox::Heap(box mem::uninitialized()));
                *trees[i as usize].lock() = Some(tree);
            }
        }
    }

    /// Filter out addresses that apply to a GiB and make them local to it
    fn localize<I>(gib: u8, usable: I) -> impl Iterator<Item=Range<usize>> + Clone
        where I: Iterator<Item=Range<usize>> + Clone
    {
        (&usable).clone()
            .filter_map(move |range| {
                let gib = ((gib as usize) << 30)..((gib as usize + 1 << 30) + 1);

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
            })
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

            if let Some(ref mut tree) = trees[index].try_lock() {
                if let Some(ref mut tree) = tree.as_mut() {
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
        let mut lock = trees[tree].lock();
        let mut tree = lock.as_mut().unwrap();

        tree.deallocate(local_ptr, order);
    }
}

enum TreeBox<'a> {
    Bootstrap(BootstrapHeapBox<'a, [Block; BLOCKS_IN_TREE]>),
    Heap(Box<[Block; BLOCKS_IN_TREE]>),
}

impl<'a> Deref for TreeBox<'a> {
    type Target = [Block; BLOCKS_IN_TREE];

    fn deref(&self) -> &[Block; BLOCKS_IN_TREE] {
        use self::TreeBox::*;
        match self {
            Bootstrap(tree_box) => tree_box,
            Heap(tree_box) => tree_box,
        }
    }
}

impl<'a> DerefMut for TreeBox<'a> {
    fn deref_mut(&mut self) -> &mut [Block; BLOCKS_IN_TREE] {
        use self::TreeBox::*;
        match self {
            Bootstrap(tree_box) => tree_box,
            Heap(tree_box) => tree_box,
        }
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
        let _tree_lock = trees[0].lock();

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

    #[test]
    fn test_init() {
        let allocator = PhysicalAllocator {
            trees: Once::new(),
        };

        allocator.init_prelim(
            9,
            iter::once(0..(9 << 30) + 1),
        );

        let trees = allocator.trees.wait().unwrap();

        assert!(trees[8].lock().is_none());
        assert!(trees[7].lock().is_some());

        allocator.init_rest(
            9,
            iter::once(0..(9 << 30) + 1),
        );

        assert!(trees[8].lock().is_some());
    }
}

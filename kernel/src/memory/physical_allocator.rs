///! A modified buddy bitmap allocator. Written originally in
/// [buddy allocator workshop](https://github.com/Restioson/buddy-allocator-workshop).
use core::{cmp, mem, ptr, ops::RangeInclusive};
use spin::{RwLock, Mutex};

use super::bootstrap_heap::{BOOTSTRAP_HEAP, BootstrapHeapBox};

/// Number of orders.
const LEVEL_COUNT: u8 = 19;
/// The maximum order.
const MAX_ORDER: u8 = LEVEL_COUNT - 1;
/// The base order size. All orders are in context of this -- i.e the size of a block of order `k`
/// is `2^(k + MIN_ORDER)`, not `2^k`.
const BASE_ORDER: u8 = 12;
/// The size as a power of two of the maximum order.
const MAX_ORDER_SIZE: u8 = BASE_ORDER + MAX_ORDER;
pub const BLOCKS_IN_TREE: usize = Tree::blocks_in_tree(LEVEL_COUNT);

/// The physical frame allocator. Requires the bootstrap heap to be initialized, or else the
/// initializer will panic.
pub static PHYSICAL_ALLOCATOR: PhysicalAllocator<'static> = PhysicalAllocator {
    trees: RwLock::new(None),
};

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

/// A tree of blocks. Contains the flat representation of the tree as a flat array
struct Tree<'a> {
    /// Flat array representation of tree. Used with the help of the `flat_tree` crate.
    flat_blocks: BootstrapHeapBox<'a, [Block; Tree::blocks_in_tree(LEVEL_COUNT)]>,
}

/// A block in the bitmap
pub struct Block {
    /// The order of the biggest block under this block - 1. 0 denotes used
    order_free: u8,
}

impl Block {
    pub fn new_free(order: u8) -> Self {
        Block {
            order_free: order + 1,
        }
    }

    pub fn new_used() -> Self {
        Block {
            order_free: 0,
        }
    }
}


impl<'a> Tree<'a> {
    const fn blocks_in_tree(levels: u8) -> usize {
        ((1 << levels) - 1) as usize
    }

    /// Creates a new tree. Panics if the bootstrap allocator has not been initialized or does not
    /// have enough memory
    pub fn new<'b>(holes: &'b [RangeInclusive<usize>]) -> Self {
        let mut flat_blocks: BootstrapHeapBox<[Block; BLOCKS_IN_TREE]> = unsafe {
            BOOTSTRAP_HEAP.allocate_zeroed().unwrap()
        };

        // First, set everything up as free
        let mut start: usize = 0;
        for level in 0..LEVEL_COUNT {
            let order = MAX_ORDER - level;
            let size = 1 << (level as usize);

            for block_index in start..(start + size) {
                flat_blocks[block_index] = Block::new_free(order);
            }

            start += size;
        }

        let mut tree = Tree {
            flat_blocks,
        };

        // Then, set blocks at order 0 (level MAX_ORDER) in the memory hole to used & set their
        // parents accordingly
        let mut address: usize = 0;

        if !holes.is_empty() {
            for block_index in (1 << MAX_ORDER)..(1 << (MAX_ORDER + 1)) {
                for hole in holes {
                    if hole.contains(&address) {
                        // Set blocks
                        tree.flat_blocks[block_index - 1] = Block::new_used();

                        // Set parents
                        tree.update_blocks_above(block_index, MAX_ORDER);
                    }
                }

                address += 1 << (BASE_ORDER);
            }
        }

        tree
    }

    #[cfg(test)]
    pub const fn blocks_in_level(order: u8) -> usize {
        (1 << (BASE_ORDER + order) as usize) / (1 << (BASE_ORDER as usize))
    }

    #[inline]
    unsafe fn block_mut(&mut self, index: usize) -> &mut Block {
        debug_assert!(index < Tree::blocks_in_tree(LEVEL_COUNT));
        self.flat_blocks.get_unchecked_mut(index)
    }

    #[inline]
    unsafe fn block(&self, index: usize) -> &Block {
        debug_assert!(index < Tree::blocks_in_tree(LEVEL_COUNT));
        self.flat_blocks.get_unchecked(index)
    }

    pub fn allocate(&mut self, desired_order: u8) -> Option<*const u8> {
        debug_assert!(desired_order <= MAX_ORDER);

        let root = unsafe { self.block_mut(0) };

        // If the root node has no orders free, or if it does not have the desired order free
        if root.order_free == 0 || (root.order_free - 1) < desired_order {
            return None;
        }

        let mut addr: u32 = 0;
        let mut node_index = 1;

        let max_level = MAX_ORDER - desired_order;

        for level in 0..max_level {
            let left_child_index = flat_tree::left_child(node_index);
            let left_child = unsafe { self.block(left_child_index - 1) };

            let o = left_child.order_free;
            // If the child is not used (o!=0) or (desired_order in o-1)
            // Due to the +1 offset, we need to subtract 1 from 0:
            // However, (o - 1) >= desired_order can be simplified to o > desired_order
            node_index = if o != 0 && o > desired_order {
                left_child_index
            } else {
                // Move over to the right: if the parent had a free order and the left didn't, the right must, or the parent is invalid and does not uphold invariants
                // Since the address is moving from the left hand side, we need to increase it
                // Block size in bytes = 2^(BASE_ORDER + order)
                // We also only want to allocate on the order of the child, hence subtracting 1
                addr += 1 << ((MAX_ORDER_SIZE - level - 1) as u32);
                left_child_index + 1
            };
        }

        let block = unsafe { self.block_mut(node_index - 1) };
        block.order_free = 0;

        self.update_blocks_above(node_index, max_level);

        Some(addr as *const u8)
    }

    #[inline]
    fn update_blocks_above(&mut self, index: usize, max_level: u8) {
        let mut node_index = index;
        // Iterate upwards and set parents accordingly
        for _ in 0..max_level {
            // Treat as right index because we need to be 0 indexed here!
            // If we exclude the last bit, we'll always get an even number (the left node while 1 indexed)
            let right_index = node_index & !1;
            node_index = flat_tree::parent(node_index);

            let left = unsafe { self.block(right_index - 1) }.order_free;
            let right = unsafe { self.block(right_index) }.order_free;

            unsafe { self.block_mut(node_index - 1) }.order_free = cmp::max(left, right);
        }
    }
}

/// Flat tree things.
///
/// # Note
/// **1 INDEXED!**
mod flat_tree {
    #[inline]
    pub fn left_child(index: usize) -> usize {
        index << 1
    }

    #[inline]
    pub fn parent(index: usize) -> usize {
        index >> 1
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;
    use super::*;

    #[test]
    fn test_holes() {
        let mut tree = Tree::new(&[0..=(0x1000 - 1)]);
        assert_eq!(tree.allocate(0), Some((1 << BASE_ORDER) as *const u8));
    }

    #[test]
    fn test_flat_tree_fns() {
        use super::flat_tree::*;
        //    1
        //  2   3
        // 4 5 6 7
        assert_eq!(left_child(1), 2);
        assert_eq!(parent(2), 1);
    }

    #[test]
    fn test_blocks_in_tree() {
        assert_eq!(Tree::blocks_in_tree(3), 1 + 2 + 4);
        assert_eq!(Tree::blocks_in_tree(1), 1);
    }

    #[test]
    fn test_tree_runs_out_of_blocks() {
        let mut tree = Tree::new(&[]);
        let max_blocks = Tree::blocks_in_level(MAX_ORDER);
        for _ in 0..max_blocks {
            assert_ne!(tree.allocate(0), None);
        }

        assert_eq!(tree.allocate(0), None);
    }

    #[test]
    fn test_init_tree() {
        let tree = Tree::new(&[]);

        // Highest level has 1 block, next has 2, next 4
        assert_eq!(tree.flat_blocks[0].order_free, 19);

        assert_eq!(tree.flat_blocks[1].order_free, 18);
        assert_eq!(tree.flat_blocks[2].order_free, 18);

        assert_eq!(tree.flat_blocks[3].order_free, 17);
        assert_eq!(tree.flat_blocks[4].order_free, 17);
        assert_eq!(tree.flat_blocks[5].order_free, 17);
        assert_eq!(tree.flat_blocks[6].order_free, 17);
    }

    #[test]
    fn test_allocate_exact() {
        let mut tree = Tree::new(&[]);
        tree.allocate(3).unwrap();

        tree = Tree::new(&[]);
        assert_eq!(tree.allocate(MAX_ORDER - 1), Some(0x0 as *const u8));
        assert_eq!(
            tree.allocate(MAX_ORDER - 1),
            Some((2usize.pow(MAX_ORDER_SIZE as u32) / 2) as *const u8)
        );
        assert_eq!(tree.allocate(0), None);
        assert_eq!(tree.allocate(MAX_ORDER - 1), None);

        tree = Tree::new(&[]);
        assert_eq!(tree.allocate(MAX_ORDER), Some(0x0 as *const u8));
        assert_eq!(tree.allocate(MAX_ORDER), None);
    }

    #[test]
    fn test_alloc_unique_addresses() {
        let max_blocks = Tree::blocks_in_level(MAX_ORDER);
        let mut seen = BTreeSet::new();
        let mut tree = Tree::new(&[]);

        for _ in 0..max_blocks {
            let addr = tree.allocate(0).unwrap();

            if seen.contains(&addr) {
                panic!("Allocator must return addresses never been allocated before!");
            } else {
                seen.insert(addr);
            }
        }
    }

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

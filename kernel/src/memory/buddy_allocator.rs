/// A block in the bitmap
pub struct Block {
    /// The order of the biggest block under this block - 1. 0 denotes used
    pub order_free: u8,
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

#[inline]
pub const fn blocks_in_tree(levels: u8) -> usize {
    ((1 << levels) - 1) as usize
}

#[inline]
pub const fn blocks_in_level(level: u8) -> usize {
    blocks_in_tree(level + 1) - blocks_in_tree(level)
}

/// Flat tree things.
///
/// # Note
/// **1 INDEXED!**
pub mod flat_tree {
    #[inline]
    pub fn left_child(index: usize) -> usize {
        index << 1
    }

    #[inline]
    pub fn parent(index: usize) -> usize {
        index >> 1
    }
}

macro_rules! buddy_allocator_bitmap_tree {
    (LEVEL_COUNT = $LEVEL_COUNT:expr, BASE_ORDER = $BASE_ORDER:expr) => {
        use $crate::memory::buddy_allocator::Block;

        pub const BLOCKS_IN_TREE: usize = $crate::memory::buddy_allocator::blocks_in_tree(
            $LEVEL_COUNT
        );
        const MAX_ORDER: u8 = $LEVEL_COUNT - 1;
        const MAX_ORDER_SIZE: u8 = $BASE_ORDER + MAX_ORDER;

        const_assert!(level_count_too_big; $LEVEL_COUNT < 128);

        /// A tree of blocks. Contains the flat representation of the tree as a flat array
        struct Tree<B>
            where B: ::core::ops::DerefMut<Target = [Block; BLOCKS_IN_TREE]>,
        {
            /// Flat array representation of tree. Used with the help of the `flat_tree` module.
            flat_blocks: B,
        }

        impl<B> Tree<B>
            where B: ::core::ops::DerefMut<Target = [Block; BLOCKS_IN_TREE]>,
        {
            pub fn new<I>(usable: I, flat_blocks: B) -> Self
                 where I: Iterator<Item=::core::ops::Range<usize>> + Clone,
            {
                use $crate::memory::buddy_allocator::blocks_in_level;
                let mut tree = Tree { flat_blocks };

                // Set blocks at order 0 (level = MAX_ORDER) in the holes to used & set
                // their parents accordingly. This is implemented by checking if the block falls
                // completely within a usable memory area.
                let mut block_begin: usize = 0;

                for block_index in (1 << MAX_ORDER)..(1 << (MAX_ORDER + 1)) {
                    let block_end = block_begin + (1 << $BASE_ORDER) - 1;

                    if !(usable.clone())
                        .any(|area| (area.contains(&block_begin) && area.contains(&block_end)))
                    {
                        unsafe { *tree.block_mut(block_index - 1) = Block::new_used(); }
                    } else {
                        unsafe { *tree.block_mut(block_index - 1) = Block::new_free(0); }
                    }

                    block_begin += 1 << ($BASE_ORDER);
                }

                let mut start: usize = 1 << (MAX_ORDER - 1);
                for level in (0..MAX_ORDER).rev() {
                    for node_index in start..(start +  blocks_in_level(level)) {
                        tree.update_block(node_index, level);
                    }

                    start >>= 1;
                }
                tree
            }

            #[inline]
            unsafe fn block_mut(&mut self, index: usize) -> &mut Block {
                #[cfg(any(debug_assertions, test))]
                return &mut self.flat_blocks[index];

                #[cfg(not(any(debug_assertions, test)))]
                self.flat_blocks.get_unchecked_mut(index)
            }

            #[inline]
            unsafe fn block(&self, index: usize) -> &Block {
                #[cfg(any(debug_assertions, test))]
                return &self.flat_blocks[index];

                #[cfg(not(any(debug_assertions, test)))]
                self.flat_blocks.get_unchecked(index)
            }

            /// Allocate a block of `desired_order` if one is available, returning a pointer
            /// relative to the tree (i.e `0` is the beginning of the tree's memory).
            pub fn allocate(&mut self, desired_order: u8) -> Option<*const u8> {
                use $crate::memory::buddy_allocator::flat_tree;

                assert!(desired_order <= MAX_ORDER);

                let root = unsafe { self.block_mut(0) };

                // If the root node has no orders free, or if it does not have at least the desired
                // order free, then no blocks are available
                if root.order_free == 0 || (root.order_free - 1) < desired_order {
                    return None;
                }

                let mut addr: u32 = 0;
                let mut node_index = 1;

                let max_level = MAX_ORDER - desired_order;

                for level in 0..max_level {
                    let left_child_index = flat_tree::left_child(
                        node_index
                    );
                    let left_child = unsafe { self.block(left_child_index - 1) };

                    let o = left_child.order_free;
                    // If the child is not used (o!=0) or (desired_order in o-1)
                    // Due to the +1 offset, we need to subtract 1 from 0.
                    // However, (o - 1) >= desired_order can be simplified to o > desired_order
                    node_index = if o != 0 && o > desired_order {
                        left_child_index
                    } else {
                        // Move over to the right: if the parent had a free order and the left didn't,
                        // the right must, or the parent is invalid and does not uphold invariants.

                        // Since the address is moving from the left hand side, we need to increase
                        // it by the size, which is 2^(BASE_ORDER + order) bytes

                        // We also only want to allocate on the order of the child, hence
                        // subtracting 1
                        addr += 1 << ((MAX_ORDER_SIZE - level - 1) as u32);
                        left_child_index + 1
                    };
                }

                let block = unsafe { self.block_mut(node_index - 1) };
                block.order_free = 0;

                self.update_blocks_above(node_index, max_level);

                Some(addr as *const u8)
            }

            /// Deallocate a block of memory from a pointer relative to the tree (e.g `0` is the
            /// beginning of the tree's memory) and the order of the block.
            #[inline]
            pub fn deallocate(&mut self, ptr: *const u8, order: u8) {
                use $crate::memory::buddy_allocator::blocks_in_tree;
                use $crate::memory::buddy_allocator::flat_tree;
                use ::core::cmp;

                assert!(order <= MAX_ORDER, "Block order > maximum order!");

                let level = MAX_ORDER - order;
                let level_offset = blocks_in_tree(level);
                let index = level_offset + ((ptr as usize) >> (order + $BASE_ORDER)) + 1;

                assert!(index < BLOCKS_IN_TREE, "Block index {} out of bounds!", index);
                assert_eq!(
                    unsafe { self.block(index - 1).order_free },
                    0,
                    "Block to free (index {}) must be used!",
                    index,
                );

                // Only if order isn't 0 we need to check the children, as blocks of order 0 have
                // no children
                if order != 0 {
                    // Treat this as a right child. It would be the left child, but since it's 1
                    // indexed it's 1 greater than the array index, and so is the right child, so
                    // they balance out.
                    let right_child = flat_tree::left_child(index);

                    // Set this block's order free to the max of both its children. If both are
                    // free, however, then this block must have its own `order` free, as the
                    // children can be merged.
                    unsafe {
                        let left = self.block(right_child - 1).order_free;
                        let right = self.block(right_child).order_free;
                        if (left == order) && (right == order) {
                            self.block_mut(index - 1).order_free = order + 1;
                        } else {
                            debug_assert!(left != 0 && right != 0, "Children must not be used!");
                            self.block_mut(index - 1).order_free = cmp::max(left, right);
                        }
                    }
                } else {
                    unsafe { self.block_mut(index - 1).order_free = 1; }
                }

                self.update_blocks_above(index, MAX_ORDER - order);
            }

            /// Update a block from its children
            #[inline]
            fn update_block(&mut self, node_index: usize, level: u8) {
                use ::core::cmp;
                use $crate::memory::buddy_allocator::flat_tree;

                assert!(
                    level != MAX_ORDER,
                    "Order 0 does not have children and thus cannot be updated from them!"
                );

                assert!(
                    node_index != 0,
                    "Node index 0 is invalid in 1 index tree!"
                );

                unsafe {
                    // The ZERO indexed left child index
                    let left_index = flat_tree::left_child(node_index) - 1;

                    let left = self.block(left_index).order_free;
                    let right = self.block(left_index + 1).order_free;
                    let order = MAX_ORDER - level;

                    if (left == order) && (right == order) {
                        // Merge blocks
                        self.block_mut(node_index - 1).order_free = order + 1;
                    } else {
                        self.block_mut(node_index - 1).order_free = cmp::max(left, right);
                    }
                }
            }

            #[inline]
            fn update_blocks_above(&mut self, index: usize, max_level: u8) {
                use $crate::memory::buddy_allocator::flat_tree;

                let mut node_index = index;
                // Iterate upwards and set parents accordingly
                for level in 0..max_level {
                    node_index = flat_tree::parent(node_index);

                    self.update_block(node_index, level);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use core::iter;
    use std::collections::BTreeSet;
    use std::mem;
    use super::*;

    buddy_allocator_bitmap_tree!(LEVEL_COUNT = 19, BASE_ORDER = 12);

    #[test]
    fn test_usable() {
        let mut tree = Tree::new(
            iter::once(0x1000..0x2001),
            unsafe { box mem::uninitialized() }
        );
        assert_eq!(tree.allocate(0), Some((1 << 12) as *const u8));
        assert_eq!(tree.allocate(0), None);
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
        assert_eq!(blocks_in_tree(3), 1 + 2 + 4);
        assert_eq!(blocks_in_tree(1), 1);
    }

    #[test]
    fn test_blocks_in_level() {
        assert_eq!(blocks_in_level(2), 4);
        assert_eq!(blocks_in_level(0), 1);
    }

    #[test]
    fn test_tree_runs_out_of_blocks() {
        let mut tree = Tree::new(
            iter::once(0..(1 << 30 + 1)),
            unsafe { box mem::uninitialized() }
        );
        let max_blocks = blocks_in_level(MAX_ORDER);

        for _ in 0..max_blocks {
            assert_ne!(tree.allocate(0), None);
        }
        assert_eq!(tree.allocate(0), None);
    }

    #[test]
    fn test_init_tree() {
        let tree = Tree::new(
            iter::once(0..(1 << 30 + 1)),
            unsafe { box mem::uninitialized() }
        );

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
        let mut tree = Tree::new(
            iter::once(0..(1 << 30 + 1)),
            unsafe { box mem::uninitialized() }
        );
        tree.allocate(3).unwrap();

        tree = Tree::new(
            iter::once(0..(1 << 30 + 1)),
            unsafe { box mem::uninitialized() }
        );
        assert_eq!(tree.allocate(MAX_ORDER - 1), Some(0x0 as *const u8));
        assert_eq!(
            tree.allocate(MAX_ORDER - 1),
            Some((2usize.pow(MAX_ORDER_SIZE as u32) / 2) as *const u8)
        );
        assert_eq!(tree.allocate(0), None);
        assert_eq!(tree.allocate(MAX_ORDER - 1), None);

        tree = Tree::new(
            iter::once(0..(1 << 30 + 1)),
            unsafe { box mem::uninitialized() }
        );
        assert_eq!(tree.allocate(MAX_ORDER), Some(0x0 as *const u8));
        assert_eq!(tree.allocate(MAX_ORDER), None);
    }


    #[test]
    fn test_free() {
        let mut tree = Tree::new(
            iter::once(0..(1 << 30 + 1)),
            unsafe { box mem::uninitialized() }
        );
        let ptr = tree.allocate(3).unwrap();
        tree.deallocate(ptr, 3);

        let ptr2 = tree.allocate(3).unwrap();
        assert_eq!(ptr2, ptr);
        tree.deallocate(ptr2, 3);

        let ptr = tree.allocate(1).unwrap();
        tree.deallocate(ptr, 1);

        let ptr = tree.allocate(0).unwrap();
        let ptr2 = tree.allocate(0).unwrap();

        tree.deallocate(ptr, 0);
        assert_eq!(tree.allocate(0).unwrap(), ptr);
        tree.deallocate(ptr2, 0);
        tree.deallocate(ptr, 0);

        assert_eq!(tree.allocate(5).unwrap(), 0x0 as *const u8);
    }

    #[test]
    fn test_alloc_unique_addresses() {
        let max_blocks = blocks_in_level(MAX_ORDER);
        let mut seen = BTreeSet::new();
        let mut tree = Tree::new(
            iter::once(0..(1 << 30 + 1)),
            unsafe { box mem::uninitialized() }
        );

        for _ in 0..max_blocks {
            let addr = tree.allocate(0).unwrap();

            if seen.contains(&addr) {
                panic!("Allocator must return addresses never been allocated before!");
            } else {
                seen.insert(addr);
            }
        }
    }
}

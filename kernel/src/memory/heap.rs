/// The base heap address.
pub const HEAP_START: usize = 0xffffffff40000000;

use core::alloc::{GlobalAlloc, Layout};
use core::{iter, mem};
use core::ptr::Unique;
use core::ops::{Deref, DerefMut};
use spin::{Once, Mutex};
use super::paging::{PAGE_TABLES, Page, PageSize, EntryFlags};
use memory::paging::PhysicalAddress;
use util;
// use ...::Block // <-- this one comes from the macro invocation below

buddy_allocator_bitmap_tree!(LEVEL_COUNT = 25, BASE_ORDER = 6);

/// Wrapper that just impls deref for a Unique.
///
/// # Safety
///
/// Safe if the Unique is valid.
struct DerefPtr<T>(Unique<T>);

impl<T> DerefPtr<T> {
    const fn new(unique: Unique<T>) -> Self {
        DerefPtr(unique)
    }
}

impl<T> Deref for DerefPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for DerefPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

pub struct Heap {
    tree: Once<Mutex<Tree<DerefPtr<[Block; BLOCKS_IN_TREE]>>>>,
}

impl Heap {
    pub const fn new() -> Self {
        Heap { tree: Once::new() }
    }
    
    /// Initializes the heap. Required for it to be usable, otherwise all of its methods will panic.
    ///
    /// # Unsafety
    ///
    /// Unsafe because `heap_tree_start` needs to be correct (unused) and well-aligned (currently
    /// non applicable as Block is a u8).
    pub unsafe fn init(&self, heap_tree_start: usize) -> usize {
        self.tree.call_once(|| {
            // Get the next page up from the given heap start
            let heap_tree_start = ((heap_tree_start / 4096) + 1) * 4096;

            // Map pages for the tree to use for accounting info
            let pages_to_map = util::round_up_divide(
                mem::size_of::<[Block; BLOCKS_IN_TREE]>() as u64,
                4096,
            );

            for page in 0..pages_to_map as usize {
                let mut table = PAGE_TABLES.lock();
                table.map(
                    Page::containing_address(heap_tree_start + (page * 4096), PageSize::Kib4),
                    EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
                    true,
                );
            }

            let tree = Tree::new(
                iter::once(0..(1 << 30 + 1)),
                DerefPtr::new(Unique::new_unchecked(heap_tree_start as *mut _)),
            );

            Mutex::new(tree)
        });

        ((heap_tree_start / 4096) + 1) * 4096
    }

    /// Allocate a block of minimum size of 4096 bytes (rounded to this if smaller) with specific
    /// requirements about where it is to be placed in physical memory.
    ///
    /// Note: `physical_begin_frame` is the frame number of the beginning physical frame to allocate
    /// memory from (i.e address / 4096).
    ///
    /// # Panicking
    ///
    /// Panics if the heap is not initialized.
    ///
    /// # Unsafety
    ///
    /// Unsafe as it remaps pages, which could cause memory unsafety if the heap is not set up
    /// correctly.
    pub unsafe fn alloc_specific(
        &self,
        physical_begin_frame: usize,
        frames: usize,
    ) -> *mut u8 {
        let mut tree = self.tree.wait().expect("Heap not initialized!").lock();
        
        let order = order(frames * 4096);
        if order > MAX_ORDER { return 0 as *mut _; }

        let ptr = tree.allocate(order);

        if ptr.is_none() { return 0 as *mut _; }

        let ptr = (ptr.unwrap() as usize + HEAP_START) as *mut u8;

        // Map pages that must be mapped
        // 6 is base order so `1 << (order + 6)`
        for page in 0..util::round_up_divide(1u64 << (order + 6), 4096) as usize {
            let page_addr = ptr as usize + (page * 4096);
            PAGE_TABLES.lock().map_to(
                Page::containing_address(page_addr, PageSize::Kib4),
                PhysicalAddress((physical_begin_frame + page) * 4096),
                EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
                true, // Do invplg
            );
        }

        ptr
    }

    /// The `dealloc` counterpart to `alloc_specific`. This function does not free the backing
    /// physical memory.
    /// 
    /// # Panicking
    ///
    /// Panics if the heap is not initialized.
    ///
      /// # Unsafety
    ///
    /// Unsafe as it unmaps pages, which could cause memory unsafety if the heap is not set up
    /// correctly.
    pub unsafe fn dealloc_specific(&self, ptr: *mut u8, frames: usize) {
        if ptr.is_null() || frames == 0 {
            return;
        }

        let order = order(frames * 4096);

        assert!(
            ptr as usize >= HEAP_START &&
                (ptr as usize) < (HEAP_START + (1 << 30)),
            "Heap object {:?} pointer not in heap!",
            ptr,
        );

        let global_ptr = ptr;
        let ptr = ptr as usize - HEAP_START;

        self.tree.wait().expect("Heap not initialized!").lock().deallocate(ptr as *mut _, order);

        // Unmap pages that have were used for this alloc
        // 6 is base order so `1 << (order + 6)`
        for page in 0..util::round_up_divide(1u64 << (order + 6), 4096) as usize {
            let page_addr = global_ptr as usize + (page * 4096);

            PAGE_TABLES.lock().unmap(
                Page::containing_address(page_addr, PageSize::Kib4),
                false, // Do not free backing memory
                true, // Do invplg
            );
        }
    }

    pub const fn tree_size() -> usize {
        mem::size_of::<[Block; BLOCKS_IN_TREE]>()
    }

    pub fn is_free(&self, ptr: *const u8, layout: Layout) {
        let order = order(layout.size());
        let global_ptr = ptr;
        let ptr = ptr as usize - HEAP_START;

        let level = MAX_ORDER - order;
        let level_offset = super::buddy_allocator::blocks_in_level(level);
        let index = level_offset + ((ptr as usize) >> (order + 6));

        debug!(
            "Heap: Block @ ptr {:?} (index {:?}) = {:?}",
            global_ptr,
            index,
            unsafe { self.tree.wait().unwrap().lock().block(index - 1).order_free }
        );
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut tree = self.tree.wait().expect("Heap not initialized!").lock();

        let order = order(layout.size());
        if order > MAX_ORDER { return 0 as *mut _ }

        let ptr = tree.allocate(order);
        if ptr.is_none() { return 0 as *mut _ }
        let ptr = (ptr.unwrap() as usize + HEAP_START) as *mut u8;

        // Map pages that have yet to be mapped
        // 6 is base order so `1 << (order + 6 - 1)`
        for page in 0..util::round_up_divide(1u64 << (order + 5), 4096) as usize {
            let mut page_tables = PAGE_TABLES.lock();

            let page_addr = ptr as usize + (page * 4096);

            let mapped = page_tables
                .walk_page_table(
                    Page::containing_address(page_addr, PageSize::Kib4)
                ).is_some();

            if !mapped {
                page_tables.map(
                    Page::containing_address(page_addr, PageSize::Kib4),
                    EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
                    false, // Do not invplg -- not an overwrite
                );
            }
        }
        ptr
    }

   unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let order = order(layout.size());

        assert!(
            ptr as usize >= HEAP_START &&
                (ptr as usize) < (HEAP_START + (1 << 30)),
            "Heap object {:?} pointer not in heap!",
            ptr,
        );

        let global_ptr = ptr;
        let ptr = ptr as usize - HEAP_START;

        self.tree.wait().expect("Heap not initialized!").lock().deallocate(ptr as *mut _, order);

        // There will only be pages to unmap which totally contained this allocation if this
        // allocation was larger or equal to the size of a page
        // TODO NO: if it happened to be on its own page we must unmap too
        if order < 12 - 6  { // log2(4096) - base order
            return;
        }

        // Unmap pages that have were only used for this alloc
        // 6 is base order, but order is + 1 (used is 0) so `1 << (order + 5)`
        for page in 0..util::round_up_divide(1u64 << (order + 5), 4096) as usize {
            let page_addr = global_ptr as usize + (page * 4096);

            PAGE_TABLES.lock().unmap(
                Page::containing_address(page_addr, PageSize::Kib4),
                true, // Free backing memory
                true, // Do invplg
            );
        }
   }
}


/// Converts log2 to order (NOT minus 1)
fn order(val: usize) -> u8 {
    if val == 0 {
        return 0;
    }

    // Calculates the integer log2 of the given input
    let mut i = val;
    let mut log2 = 0;
    while i > 0 {
        i >>= 1;
        log2 += 1;
    }

    let log2 = log2;

    if log2 > 6 {
        log2 - 6
    } else {
        0
    }
}
